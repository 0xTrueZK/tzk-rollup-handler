use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::{Client, types::Tag};
use serde::Deserialize;
use warp::{http::StatusCode, Filter, Rejection, Reply, cors};

#[derive(Debug, Deserialize)]
struct InstanceRequest {
    ami_id: String,
    instance_name: String,  // Field for the instance name
}

async fn create_instance(ami_id: String, instance_name: String) -> Result<impl Reply, Rejection> {
    let region_provider = RegionProviderChain::default_provider();
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let result = client.run_instances()
        .image_id(ami_id)
        .instance_type(aws_sdk_ec2::types::InstanceType::T4gLarge)
        .min_count(1)
        .max_count(1)
        .key_name("true-net-agnish")
        .tag_specifications(aws_sdk_ec2::types::TagSpecification::builder()
            .resource_type(aws_sdk_ec2::types::ResourceType::Instance)
            .tags(Tag::builder().key("Name").value(instance_name).build())
            .build())
        .send()
        .await;

    match result {
        Ok(output) => {
            if let Some(instances) = output.instances {
                if let Some(instance) = instances.first() {
                    if let Some(instance_id) = &instance.instance_id {
                        return Ok(warp::reply::with_status(format!("{}", instance_id), StatusCode::OK));
                    }
                }
            }
            Err(warp::reject::reject())
        },
        Err(e) => {
            eprintln!("Error creating instance: {}", e);
            Err(warp::reject::reject())
        },
    }
}

async fn create_instance_handler(body: InstanceRequest) -> Result<impl Reply, Rejection> {
    create_instance(body.ami_id, body.instance_name).await
}

fn cors_filter() -> cors::Cors {
    warp::cors()
        .allow_origins(vec!["http://localhost:3010", "http://staging-app.truezk.com", "http://app.truezk.com" , "https://staging-app.truezk.com", "https://app.truezk.com"])
        .allow_headers(vec!["User-Agent", "Content-Type"])
        .allow_methods(vec!["POST"])
        .build()
}

#[tokio::main]
async fn main() {
    let instance_route = warp::post()
        .and(warp::path("create_instance"))
        .and(warp::body::json())
        .and_then(create_instance_handler)
        .with(cors_filter());

    warp::serve(instance_route)
        .run(([0, 0, 0, 0], 8080))
        .await;
}