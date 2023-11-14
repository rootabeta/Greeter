use std::collections::HashSet;
use std::fs;
use std::fs::read_to_string;
use toml::Table;

mod api;

// Ripped right out of Ruststone lmfao
fn load_config(config_file: &str) -> Table {
    let config: Table = toml::from_str(config_file).unwrap();
    return config;
}

fn load_nations_from_file(nations_file: &str) -> Result<HashSet<String>, std::io::Error> {
    let mut nations_list = HashSet::new();

    // Returns an Err if the file does not exist
    // The program should recognize if this is the case and
    // Initialize the file if this occurs
    for line in read_to_string(nations_file)?.lines() {
        nations_list.insert(api::canonicalize(line.to_string()));
    }

    Ok(nations_list)
}
fn count_new_arrivals(old_nations: HashSet<String>, new_nations: &Vec<String>) -> u64 {
    let mut arrival_count: u64 = 0;
    for nation in new_nations.iter() {
        if !old_nations.contains(nation) {
            arrival_count += 1;
        }
    }
    return arrival_count;
}

fn address_new_arrivals(old_nations: HashSet<String>, new_nations: &Vec<String>) -> String {
    let arrivaltext: String = String::from("New Arrivals");
    let mut arrival_string = format!("[spoiler={arrivaltext}]");
    for nation in new_nations.iter() {
        if !old_nations.contains(nation) {
            // I am so tired of fighting with format!
            arrival_string.push_str("[nation]");
            arrival_string.push_str(nation);
            arrival_string.push_str("[/nation] ");
        }
    }
    arrival_string.push_str("[/spoiler]");

    return arrival_string;
}

fn save_nations_to_file(
    nations_file: &str,
    new_nations: Vec<String>,
) -> Result<String, std::io::Error> {
    let mut file_content: String = String::new();
    for nation in new_nations.iter() {
        file_content.push_str(nation);
        file_content.push_str("\n");
    }
    let _ = fs::write(nations_file, file_content)?;

    Ok(nations_file.to_string())
}

fn build_message(template: String, new_arrivals: String) -> String {
    let message = str::replace(template.as_str(), "%ARRIVALS%", new_arrivals.as_str());

    return message;
}

fn main() {
    let config_file = match fs::read_to_string("config.toml") {
        Ok(config) => config,
        Err(_) => panic!("Failed to read config.toml - please check that the file exists."),
    };

    let config = load_config(config_file.as_str());

    let main_nation: String = String::from(config["config"]["main_nation"].as_str().unwrap());
    let region_name: String = String::from(config["config"]["region"].as_str().unwrap());
    let nation: String = String::from(config["config"]["nation"].as_str().unwrap());
    let password: String = String::from(config["config"]["password"].as_str().unwrap());
    let rmb_template: String = String::from(config["config"]["message"].as_str().unwrap());

    // Minimum new nations. 0 to disable check. Which is stupid.
    let min_nations: u64 = config["config"]["min_nations"].as_integer().unwrap() as u64;
    let nations_file = config["config"]["nations_file"].as_str().unwrap();

    let mut api_client = api::build_client(main_nation);
    println!("Loading {nations_file}");

    let old_nations = load_nations_from_file(nations_file);

    // Failed to load file
    if old_nations.is_err() {
        println!("Failed to load nations file. Initializing.");
        let new_nations = match api_client.get_nations(&region_name) {
            Ok(nations) => nations,
            Err(e) => panic!("Experienced error while fetching nations: {e}"),
        };

        let _ = match save_nations_to_file(nations_file, new_nations) {
            Ok(_) => println!("Saved nations to {nations_file}."),
            Err(e) => panic!("Experienced error while saving to file: {e}"),
        };

        return;
    }

    println!("Fetching current nations.");
    let new_nations = match api_client.get_nations(&region_name) {
        Ok(nations) => nations,
        Err(e) => panic!("Experienced error while fetching nations: {e}"),
    };

    // At this point, we know the old_nations loaded :3
    let old_nations = old_nations.unwrap();

    let new_arrival_count = count_new_arrivals(old_nations.clone(), &new_nations);

    // Abort if nation count is insufficient
    if new_arrival_count < min_nations {
        println!("Insufficient number of nations to justify sending message ({new_arrival_count}/{min_nations})");
        return;
    }

    let new_arrivals = address_new_arrivals(old_nations, &new_nations);

    let message = build_message(rmb_template, new_arrivals);
    println!("Built message: \n{message}");

    let _ = match api_client.login(&nation, password) {
        Ok(_) => println!("Logged into {nation} successfully."),
        Err(e) => panic!("Failed to log into {nation}! Check your credentials.\nError: {e}"),
    };

    let _ = match api_client.send_rmb(&region_name, message) {
        Ok(_) => println!("Successfully sent RMB post."),
        Err(e) => panic!("Sending RMB post failed with error {e}"),
    };

    let _ = match save_nations_to_file(nations_file, new_nations) {
        Ok(_) => println!("Saved nations to {nations_file}."),
        Err(e) => panic!("Experienced error while saving to file: {e}"),
    };
}
