use serde::Deserialize;
use serde_xml_rs::from_str;
use ureq::{Agent, Error};
use std::thread::sleep;
use std::time::Duration;

pub fn canonicalize(string: String) -> String { 
    let mut output = string.clone();
    output.make_ascii_lowercase();
    return str::replace(output.as_str(), " ", "_");
}

#[derive(Deserialize)]
struct Nations { 
    #[serde(alias="NATIONS")]
    nations: String
}

pub struct APIClient { 
    agent: Agent,
    user_agent: String,
    nation: String,
    password: String, 
    x_pin: u64,
}

#[derive(Deserialize)]
pub struct Token { 
    #[serde(alias="SUCCESS")]
    token: String
}

impl APIClient { 
    // TODO
    pub fn login(&mut self, nation: &String, password: String) -> Result<u64, Error> { 
        self.nation = nation.to_string(); 
        self.password = password;

        let nation = canonicalize(nation.to_string());
        let url: String = format!("https://www.nationstates.net/cgi-bin/api.cgi?nation={nation}&q=ping");

        self.x_pin = self.agent
            .get(url.as_str())
            .set("User-Agent", self.user_agent.as_str())
            .set("X-Password", self.password.as_str())
            .call()?
            .header("X-Pin").expect("Failed to log in!")
            .trim()
            .parse::<u64>()
            .unwrap();

        sleep(
            Duration::from_millis(750)
        );

        Ok(self.x_pin)
    }

    pub fn get_nations(&self, region_name: &String) -> Result<Vec<String>, Error> { 
        // Fetch nation list, propogating error upwards
        const SHARD: &str = "nations";
        let region = canonicalize(region_name.to_string());
        let url: String = format!("https://www.nationstates.net/cgi-bin/api.cgi?region={region}&q={SHARD}");
        let response: String = self.agent
            .get(url.as_str())
            .set("User-Agent", self.user_agent.as_str())
            .call()?
            .into_string()
            .unwrap();

        // Time delay
        sleep(
            Duration::from_millis(750)
        );

        // Load into string
        let nation_class: Nations = from_str(
            response.as_str()
        ).unwrap();

        // Parse into a Vector
        let mut nation_list = Vec::new();
        for nation in nation_class.nations.split(":") { 
            nation_list.push(canonicalize(nation.to_string()));
        }

        Ok(nation_list)
    }

    // TODO
    pub fn send_rmb(&self, region_name: &String, message: String) -> Result<String, Error> { 
        let url: String = "https://www.nationstates.net/cgi-bin/api.cgi".to_string();
        let xpin: String = self.x_pin.to_string();
        let token_raw = self.agent
            .post(&url)
            .set("User-Agent", self.user_agent.as_str())
            .set("X-Pin", &xpin)
            .send_form(&[
                    ("nation", &self.nation),
                    ("region", &region_name),
                    ("c", "rmbpost"),
                    ("text", &message),
                    ("mode", "prepare"),
            ])?
            .into_string()?;

        let token_struct: Token = from_str(
            token_raw.as_str()
        ).unwrap();

        sleep(
            Duration::from_millis(750)
        );

        let resp = self.agent
            .post(&url)
            .set("User-Agent", self.user_agent.as_str())
            .set("X-Pin", &xpin)
            .send_form(&[
                    ("nation", &self.nation),
                    ("region", &region_name),
                    ("c", "rmbpost"),
                    ("text", &message),
                    ("mode", "execute"),
                    ("token", &token_struct.token),
            ])?
            .into_string()
            .unwrap();

        sleep(
            Duration::from_millis(750)
        );

        Ok(resp)
    }
}

pub fn build_client(main_nation: String) -> APIClient { 
    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let api_client = APIClient{
        agent: agent, 
        user_agent: format!(
            "Greeter/{}; Developed by nation=Volstrostia; In use by {}",
            env!("CARGO_PKG_VERSION"),
            canonicalize(main_nation.clone()),
        ),
        nation: String::new(),
        password: String::new(),
        x_pin: 0,
    };
    return api_client;
}
