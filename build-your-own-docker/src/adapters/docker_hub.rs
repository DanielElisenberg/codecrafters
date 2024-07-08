use anyhow::Context;
use reqwest::Client;

#[derive(serde::Deserialize, Debug)]
struct AuthResponse {
    token: String,
}

#[derive(serde::Deserialize, Debug)]
struct FsLayer {
    #[serde(rename = "blobSum")]
    pub blob_sum: String,
}

#[derive(serde::Deserialize, Debug)]
struct Manifest {
    #[serde(rename = "fsLayers")]
    pub fs_layers: Vec<FsLayer>,
}

const DOCKER_AUTH_URL: &str = "https://auth.docker.io/token";
const DOCKER_SERVICE_URL: &str = "registry.docker.io";
const DOCKER_REGISTRY_V2_URL: &str = "https://registry.hub.docker.com/v2/";

async fn get_auth_token(image_name: &str) -> anyhow::Result<String> {
    Ok(Client::new()
        .get(&format!(
            "{}?service={}&scope=repository:library/{}:pull",
            DOCKER_AUTH_URL, DOCKER_SERVICE_URL, image_name
        ))
        .send()
        .await
        .with_context(|| format!("Failed to fetch auth token for image '{}'", image_name))?
        .json::<AuthResponse>()
        .await?
        .token)
}

async fn get_manifest(
    image_name: &str,
    image_tag: &str,
    auth_token: &str,
) -> anyhow::Result<Manifest> {
    Ok(Client::new()
        .get(&format!(
            "{}library/{}/manifests/{}",
            DOCKER_REGISTRY_V2_URL, image_name, image_tag
        ))
        .bearer_auth(auth_token)
        .send()
        .await
        .with_context(|| format!("Failed to fetch manifest for image '{}'", image_name))?
        .json::<Manifest>()
        .await?)
}

async fn get_blob_as_bytes(
    sha: &str,
    image_name: &str,
    auth_token: &str,
) -> anyhow::Result<bytes::Bytes> {
    Ok(Client::new()
        .get(&format!(
            "{}library/{}/blobs/{}",
            DOCKER_REGISTRY_V2_URL, image_name, sha
        ))
        .bearer_auth(auth_token)
        .send()
        .await
        .with_context(|| format!("Failed to fetch blob for image '{}'", image_name))?
        .bytes()
        .await?)
}

pub async fn get_image_blobs_as_bytes(
    image_name: &str,
    image_tag: &str,
) -> anyhow::Result<Vec<bytes::Bytes>> {
    // Get an authentication token for the docker API
    let auth_token = get_auth_token(image_name).await?;
    // Get the v2 manifest for supplied image using the auth token
    let manifest = get_manifest(image_name, image_tag, &auth_token).await?;
    let mut blobs: Vec<bytes::Bytes> = vec![];
    for layer in manifest.fs_layers.into_iter() {
        blobs.push(get_blob_as_bytes(&layer.blob_sum, image_name, &auth_token).await?);
    }
    Ok(blobs)
}
