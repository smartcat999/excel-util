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
            .proxy(proxy.clone())
            .build()
            .unwrap();
        let user_agent = HttpClient::read_user_agent("");

        HttpClient {client, user_agent }
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
        url: &str
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


pub struct AsyncHttpClient {
    async_client: reqwest::Client,
    user_agent: Vec<String>,
}

impl AsyncHttpClient {
    pub fn new() -> AsyncHttpClient {
        let proxy = reqwest::Proxy::http("http://116.9.163.205:58080").unwrap();
        let user_agent = crate::command::cve::api::base::HttpClient::read_user_agent("");
        let async_client = reqwest::ClientBuilder::new()
            .pool_max_idle_per_host(50)
            .pool_idle_timeout(Duration::from_secs(2))
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(10))
            .proxy(proxy)
            .build()
            .unwrap();
        AsyncHttpClient {async_client, user_agent}
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

    pub async fn get(&self,
                           url: &str) -> Result<reqwest::Response,  Box<dyn std::error::Error>> {
        let user_agent = match self.random_user_agent() {
            Some(v) => v,
            None => "",
        };

        let resp = self
            .async_client
            .get(url)
            .header(USER_AGENT, HeaderValue::from_str(user_agent).unwrap())
            .send().await?;
        Ok(resp)
    }
}

impl Default for AsyncHttpClient {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get() -> Result<(), Box<dyn std::error::Error>> {
        let http_client = HttpClient::new();
        let resp = http_client.get("https://www.baidu.com")?.text()?;
        println!("{:#?}", resp);
        Ok(())
    }

    #[test]
    fn test_async_get() -> Result<(), Box<dyn std::error::Error>> {
        let http_client = HttpClient::new();
        tokio_test::block_on(async {
            let resp = http_client.async_get("https://www.baidu.com").await?.text().await?;
            println!("{:#?}", resp);
            Ok(())
        })
    }
}
