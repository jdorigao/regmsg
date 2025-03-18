use swayipc::{Connection, Output};

pub fn wayland_list_modes() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            println!("{}x{}@{}Hz | Current Mode", current_mode.width, current_mode.height, current_mode.refresh / 1000);
        }
        for mode in output.modes {
            println!("{}x{}@{}Hz", mode.width, mode.height, mode.refresh / 1000);
        }
    }

    Ok(())
}

pub fn wayland_list_outputs() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        println!("{}", output.name);
    }

    Ok(())
}

pub fn wayland_current_mode() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            println!("{}x{}@{}Hz", current_mode.width, current_mode.height, current_mode.refresh / 1000);
        }
    }

    Ok(())
}

pub fn wayland_current_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(_current_mode) = output.current_mode {
            println!("{}", output.name);
        }
    }

    Ok(())
}

pub fn wayland_current_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            println!("{}x{}", current_mode.width, current_mode.height);
        }
    }

    Ok(())
}

pub fn wayland_get_refresh_rate() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new()?;
    let outputs: Vec<Output> = connection.get_outputs()?;

    for output in outputs {
        if let Some(current_mode) = output.current_mode {
            println!("{}Hz", current_mode.refresh / 1000);
        }
    }

    Ok(())
}
