use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::{config::Region, meta::PKG_VERSION, types::Tag, Client, Error};
use serde::Deserialize;
use warp::{http::StatusCode, Filter, Rejection, Reply, cors};

#[derive(Debug, Deserialize)]
struct InstanceRequest {
    ami_id: String,
}

async fn create_instance(ami_id: String) -> Result<impl Reply, Rejection> {
    let region_provider = RegionProviderChain::default_provider();
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let result = client.run_instances()
        .image_id(ami_id)
        .instance_type(aws_sdk_ec2::types::InstanceType::T4gXlarge)
        .min_count(1)
        .max_count(1)
        .send()
        .await;

    match result {
        Ok(_) => Ok(warp::reply::with_status("Instance created successfully", StatusCode::OK)),
        Err(e) => {
            eprintln!("Error creating instance: {}", e);
            Err(warp::reject::reject())
        },
    }

}

async fn create_instance_handler(body: InstanceRequest) -> Result<impl Reply, Rejection> {
    create_instance(body.ami_id).await
}

fn cors_filter() -> cors::Cors {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "User-Agent",
            "Content-Type",
        ])
        .allow_methods(vec!["POST"])
        .build()
}

#[tokio::main]
async fn main() {

    let instance_route = warp::post()
        .and(warp::path("create_instance"))
        .and(warp::body::json())
        .and_then(create_instance_handler)
        .with(cors_filter()); // Apply CORS settings here

    warp::serve(instance_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
