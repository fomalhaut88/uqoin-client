use crate::appdata::load_with_password;


pub fn list() -> std::io::Result<()> {
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;
    for validator in appdata.list_validators() {
        println!("{}", validator);
    }
    Ok(())
}


pub fn add(validator: &str) -> std::io::Result<()> {
    let (mut appdata, password) = load_with_password()?;
    appdata.check_not_empty()?;
    if appdata.add_validator(validator.to_string()) {
        appdata.save(&password)?;
        println!("New node added successfully.");
    } else {
        println!("This node already exists.");
    }
    Ok(())
}


pub fn remove(validator: &str) -> std::io::Result<()> {
    let (mut appdata, password) = load_with_password()?;
    appdata.check_not_empty()?;
    if appdata.remove_validator(validator) {
        appdata.save(&password)?;
        println!("The node removed successfully.");
    } else {
        println!("This node does not exist.");
    }
    Ok(())
}


pub fn r#move(validator: &str, pos: usize) -> std::io::Result<()> {
    let (mut appdata, password) = load_with_password()?;
    appdata.check_not_empty()?;
    if appdata.move_validator(validator, pos) {
        appdata.save(&password)?;
        println!("The node moved successfully.");
    } else {
        println!("This node does not exist.");
    }
    Ok(())
}


pub fn default() -> std::io::Result<()> {
    let (mut appdata, password) = load_with_password()?;
    appdata.check_not_empty()?;
    appdata.set_default_validators();
    appdata.save(&password)?;
    println!("Default nodes have been restored.");
    Ok(())
}


pub fn fetch(validator: Option<&str>) -> std::io::Result<()> {
    // Get appdata by password
    let (mut appdata, password) = load_with_password()?;
    appdata.check_not_empty()?;

    // Get validators to try
    let validators = if let Some(validator) = validator {
        vec![validator.to_string()]
    } else {
        appdata.list_validators().to_vec()
    };

    // Iterate validators
    for validator in validators.iter() {
        println!("Fetching from {}", validator);

        // Fetching the nodes
        if let Ok(nodes) = request_nodes(validator) {
            for node in nodes.iter() {
                // Add new node
                if appdata.add_validator(node.clone()) {
                    println!("Added node: {}", node);
                }
            }
        } else {
            println!("Failed to request {}", validator);
        }
    }

    // Save appdata
    appdata.save(&password)?;

    Ok(())
}


fn request_nodes(validator: &str) -> std::io::Result<Vec<String>> {
    let url = format!("{}/node/list", validator);
    let resp = reqwest::blocking::get(url.clone())
        .map_err(|_| std::io::Error::new(
            std::io::ErrorKind::NotFound.into(), url
        ))?;
    let text = resp.text().unwrap();
    Ok(serde_json::from_str(&text)?)
}
