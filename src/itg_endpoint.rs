use anyhow::Result;
use proto::{simfile_management_client::SimfileManagementClient, AddSongResponse};

pub mod proto {
    tonic::include_proto!("itg_buddy");
}
pub struct ItgEndpoint {
    responder: SimfileManagementClient<tonic::transport::Channel>,
}
impl ItgEndpoint {
    pub async fn new(url: &str) -> Result<ItgEndpoint> {
        Ok(ItgEndpoint {
            responder: SimfileManagementClient::connect(url.to_owned()).await?,
        })
    }
    pub async fn add_song(mut self, path_or_url: &str, overwrite: bool) -> Result<AddSongResponse> {
        let req = proto::AddSongRequest {
            path_or_url: path_or_url.to_owned(),
            overwrite,
        };
        Ok(self
            .responder
            .add_song(tonic::Request::new(req))
            .await?
            .into_inner())
    }
}
