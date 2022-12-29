use std::{
    env,
    fmt::{self, Display},
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::Command,
    str::FromStr,
};

extern crate dirs;

fn main() {
    let mut profile = String::new();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Choose your profile mode (home/work): ");
        io::stdin()
            .read_line(&mut profile)
            .expect("failed to read profile from readline");
    } else {
        profile = args[1].to_owned();
    }

    match profile.trim() {
        n @ ("work" | "home") => NpmRegistry::default().set_profile(
            ProfileType::from_str(n).expect("profile type only supports either `work` or `home`"),
        ),
        v => println!(
            "type `npm-registry work` or `npm-registry home` to switch npm registry. Got: {v}"
        ),
    }
}

enum ProfileType {
    Work,
    Home,
}

impl Display for ProfileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Home => write!(f, "home"),
            Self::Work => write!(f, "work"),
        }
    }
}

impl FromStr for ProfileType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "home" => Ok(Self::Home),
            "work" => Ok(Self::Work),
            _ => Err(()),
        }
    }
}

struct NpmRegistry {
    origin: String,
}

impl Default for NpmRegistry {
    fn default() -> Self {
        NpmRegistry {
            origin: String::from("https://registry.npmjs.org/"),
        }
    }
}

impl NpmRegistry {
    fn set_profile(&self, profile: ProfileType) {
        let registry = match profile {
            ProfileType::Home => self.origin.to_owned(),
            ProfileType::Work => Self::get_work_registry(),
        };

        let output = Command::new("npm")
            .arg("config")
            .arg("set")
            .arg("registry")
            .arg(registry)
            .output()
            .expect("failed to switch npm registry.");

        match output.status.code() {
            Some(0) => println!(
                "Switching registry to {} successfully.\n{}",
                profile,
                String::from_utf8(output.stdout).unwrap()
            ),
            Some(_) | None => println!("Error: {}", String::from_utf8(output.stderr).unwrap()),
        }
    }

    fn get_work_registry() -> String {
        let work: String;

        if let Some(home_dir) = dirs::home_dir() {
            let config_path = Path::new(&home_dir).join(".config/npm-registry.txt");
            if config_path.exists() {
                work = match fs::read_to_string(&config_path) {
                    Ok(config) => config,
                    Err(_) => panic!("read config file {} failed", &config_path.display()),
                };
            } else {
                work = prompt_input_registry();
                let mut file = File::create(config_path).unwrap();
                file.write_all(work.as_bytes()).unwrap();
            }
        } else {
            work = prompt_input_registry();
        }

        work.to_owned()
    }
}

fn prompt_input_registry() -> String {
    let mut registry = String::new();

    println!("Please type your registry: ");
    io::stdin()
        .read_line(&mut registry)
        .expect("failed to read registry from readline");

    registry.to_owned()
}
