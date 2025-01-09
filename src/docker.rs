use bollard::Docker;
use bollard::network::CreateNetworkOptions;

struct DockerInstance{
    docker:Docker
}

impl DockerInstance{
    pub fn new() -> DockerInstance {
        let docker = Docker::connect_with_local_defaults().unwrap();
        DockerInstance{
            docker
        }
    }
    fn create_network(self){
        todo!("create network")
    }
}