use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use bitcoincore_rpc::Client as BitcoinClient;
use bitcoincore_rpc::{Auth, RpcApi};
use minijinja::{context, Environment, Source};
use once_cell::sync::Lazy;
use qrcode_generator::QrCodeEcc;
use serde::Deserialize;

static ENV: Lazy<Environment<'static>> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set_source(Source::from_path("templates"));
    env
});
pub struct UIError(pub StatusCode, pub String);

impl From<anyhow::Error> for UIError {
    fn from(error: anyhow::Error) -> Self {
        UIError(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
    }
}

impl IntoResponse for UIError {
    fn into_response(self) -> Response {
        let UIError(status, msg) = self;
        (status, msg).into_response()
    }
}

fn bitcoin_client() -> anyhow::Result<BitcoinClient> {
    let url = "localhost:18443";
    let auth = Auth::UserPass("bitcoin".to_string(), "bitcoin".to_string());
    let client = ::bitcoincore_rpc::Client::new(&url, auth).map_err(anyhow::Error::from)?;
    Ok(client)
}

fn block_height() -> anyhow::Result<u64> {
    let client = bitcoin_client()?;
    let height = client.get_blockchain_info()?.blocks;
    Ok(height)
}

fn generate_address() -> anyhow::Result<String> {
    let client = bitcoin_client()?;
    let address = client.get_new_address(None, None)?.to_string();
    Ok(address)
}

async fn home() -> anyhow::Result<Html<String>, UIError> {
    let template = ENV.get_template("index.html").unwrap();
    let connect_str =
        std::env::var("FM_CONNECT_STRING").expect("FM_CONNECT_STRING environment variable not set");
    let height = block_height()?;
    let r = template
        .render(
            context!(connect_str => connect_str, pay_result=> Some("pay_result"), invoice => Some("pay_result"), height => height, address => Some("bc1...")),
        )
        .unwrap();
    Ok(Html(r))
}

#[derive(Deserialize, Debug, Clone)]
pub struct PostHomeForm {
    address: bool,
}

// async fn post_home(Form(form): Form<PostHomeForm>) -> anyhow::Result<Redirect, UIError> {
//     println!("found form: {form:?}");
//     Ok(Redirect::to("/"))
// }

async fn post_home() -> anyhow::Result<Redirect, UIError> {
    // println!("found form: {form:?}");
    Ok(Redirect::to("/"))
}

async fn qr(Path((string,)): Path<(String,)>) -> impl IntoResponse {
    let png_bytes: Vec<u8> = qrcode_generator::to_png_to_vec(string, QrCodeEcc::Low, 1024).unwrap();
    ([(axum::http::header::CONTENT_TYPE, "image/png")], png_bytes)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Make sure environment variables are set
    std::env::var("FM_CONNECT_STRING").expect("FM_CONNECT_STRING environment variable not set");

    // build our application with a single route
    let app = Router::new()
        .route("/", get(home))
        .route("/", post(post_home))
        .route("/qr/:string", get(qr));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
