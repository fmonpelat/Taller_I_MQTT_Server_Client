use std::net::{TcpStream};
use std::io::Result;
pub struct Client<'a> {
  server_host: &'a str, 
  server_port: &'a str,
    
}

impl<'a> Client<'a> {
  pub fn new(server_host: &'a str, server_port: &'a str) -> Self {
    Client{
      server_host: server_host,
      server_port: server_port,
    }
  }

  pub fn connect(&self)-> Result<TcpStream> {
    TcpStream::connect(String::from(self.server_host) + ":" + self.server_port)
  }

  pub fn get_server_port(self)-> String{
    String::from(self.server_port)
  }
}
trait Mqtt{
  fn publish();
  fn suscribe();
}

impl Mqtt for Client<'_>{
  fn publish() {
      
  }
  
  fn suscribe(){

  }
}
