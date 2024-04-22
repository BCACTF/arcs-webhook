use actix_web::{body::{to_bytes, BoxBody}, dev::ServiceResponse};
use assert_json::Value;
use serde::Serialize;

pub async fn try_parse_json(res: ServiceResponse<BoxBody>) -> Result<Value, String> {
    let body = res.into_body();
    let body = to_bytes(body).await;
    assert!(body.is_ok());

    let body = body.map_err(|e| e.to_string())?;
    let body: Vec<u8> = body.into_iter().collect();
    let body = String::from_utf8_lossy(&body).trim().to_string();
    
    serde_json::from_str::<Value>(&body).ok().ok_or(body)
}

pub fn post_req(body: impl Serialize, from_deploy: bool) -> actix_web::test::TestRequest {
    use actix_web::test::TestRequest;

    let auth = (
        "Authorization",
        format!(
            "Bearer {}",
            if from_deploy {
                "D".repeat(64)
            } else {
                "F".repeat(64)
            }
        ),
    );

    TestRequest::post()
        .uri("/")
        .insert_header(auth)
        .set_json(body)
}

pub async fn exec_post(
    body: impl Serialize,
    from_deploy: bool,
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
    label: &str
) -> Value {
    let req = post_req(body, from_deploy);
    let res = req.send_request(app).await;
    assert_eq!(res.status().as_u16(), 200, "{label}: Response body: {:?}", try_parse_json(res).await);
    
    let res = try_parse_json(res).await;
    assert!(res.is_ok(), "{label}: {}", res.err().unwrap());
    res.unwrap()
}
