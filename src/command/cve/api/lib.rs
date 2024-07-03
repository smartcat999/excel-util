use std::{
    collections::HashMap,
};
use std::future::Future;

pub trait CveApi: Sync + Send {
    fn query(&self, cve_id: &str) -> Box<dyn Cve>;
    fn id(&self) -> String;
}

pub trait Cve {
    fn to_json(&self) -> String;
    fn get(&self, key: &str) -> String;
}

pub struct CveApis {
    apis: HashMap<String, Box<dyn CveApi>>
}

impl CveApis {
    pub fn new() -> CveApis {
        let apis: HashMap<String, Box<dyn CveApi>> = HashMap::new();
        CveApis {
            apis,
        }
    }

    pub fn register(&mut self, cve_api: Box<dyn CveApi>) {
        let key = cve_api.id();

        Ok((&mut self.apis, key, cve_api))
            .and_then(|(x, y, z)| CveApis::insert(x, y, z))
            .unwrap();
    }

    fn insert(
        x: &mut HashMap<String, Box<dyn CveApi>>,
        key: String,
        cve_api: Box<dyn CveApi>,
    ) -> Result<(), ()> {
        // if !x.contains_key(&key) {
        //     x.insert(key, cve_api);
        // }
        x.entry(key).or_insert(cve_api);
        Ok(())
    }

    pub fn invoke(&self, key: &str, id: &str) -> Box<dyn Cve> {
        self.apis.get(key).unwrap().query(id)
    }
}

impl Default for CveApis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    // use super::*;

    #[test]
    fn test_add() {}
}
