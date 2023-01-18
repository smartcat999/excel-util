use std::time::Duration;
extern crate rand;

use rand::Rng;
use reqwest::header::{HeaderValue, USER_AGENT};
pub struct HttpClient {
    client: reqwest::blocking::Client,
    user_agent: Vec<String>,
}

impl HttpClient {
    pub fn new() -> HttpClient {
        let proxy = reqwest::Proxy::http("http://116.9.163.205:58080").unwrap();
        let client = reqwest::blocking::Client::builder()
            .pool_max_idle_per_host(50)
            .pool_idle_timeout(Duration::from_secs(2))
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(10))
            .proxy(proxy)
            .build()
            .unwrap();
        let user_agent = HttpClient::read_user_agent("");
        HttpClient { client, user_agent }
    }

    #[allow(unused)]
    fn mock(&self) -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get("https://www.baidu.com")?.text()?;
        println!("{:#?}", resp);
        Ok(())
    }

    fn read_user_agent(path: &str) -> Vec<String> {
        let mut target = "user_agent.txt";
        if path.trim() != "" {
            target = path;
        }
        let data = std::fs::read_to_string(target).unwrap();
        let user_agent: Vec<String> = data
            .trim()
            .split('\n')
            .map(|x| x.trim().trim_matches('\n').to_string())
            .collect();
        user_agent
    }

    fn random_user_agent(&self) -> Option<&String> {
        let max = self.user_agent.len();
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..max);
        // println!("Integer: {}", index);
        self.user_agent.get(index)
    }

    pub fn get(
        &self,
        url: &str,
    ) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
        let user_agent = match self.random_user_agent() {
            Some(v) => v,
            None => "",
        };

        let resp = self
            .client
            .get(url)
            .header(USER_AGENT, HeaderValue::from_str(user_agent).unwrap())
            .send()?;
        Ok(resp)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get() {
        let http_client = HttpClient::new();
        let ret = http_client.mock();
        println!("{:#?}", ret);
    }
}
