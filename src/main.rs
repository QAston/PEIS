// program loads portable_env.toml from current directory
// portable_env.toml describes actions to take when command is executed
// to use this for libraries use https://gcc.gnu.org/onlinedocs/gcc/Environment-Variables.html
// LIBRARY_PATH, CPATH
extern crate toml;
extern crate docopt;

use serde_derive::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use docopt::Docopt;

static USAGE: &'static str = "
Usage: portable_env [options]

Options:
    --config=FILE  Location of the config file. [default: ./portable_env.toml]
    --output=DIR  Where to put output script directories. [default: .]
";

static AUTOREMOVE_MARKER: &'static str = "this-file-is-marked-for-removal-on-generation";

#[derive(Clone,Copy)]
#[derive(Deserialize)]
#[allow(non_camel_case_types)]
enum ModType {
    PREPEND_PATH,
    APPEND_PATH,
    SET,
    PATH
}

#[derive(Clone, Copy)]
enum EnvType {
    CMD,
    BASH,
    POWERSHELL
}

fn generate_fix_path(path: &str, t: EnvType) -> String {
    match t {
        EnvType::CMD | EnvType::POWERSHELL => format!("{}", path),
        EnvType::BASH => format!("`cygpath -p  \"{}\"`", escape_bash_vars(path))
    }
}

fn escape_bash_vars(s: &str) -> String {
    s.replace("$", "\\$")
}

fn generate_get_env(name: &str, e: EnvType) -> String {
    match e {
        EnvType::CMD => format!("%{}%", name),
        EnvType::BASH  => format!("${{{}}}", name),
        EnvType::POWERSHELL => format!("${{env:{}}}", name)
    }
}

fn generate_separator(e: EnvType) -> &'static str {
    match e {
        EnvType::CMD => ";",
        EnvType::BASH  => ":",
        EnvType::POWERSHELL => ";"
    }
}

fn transform_vars(value: &str, e: EnvType) -> String {
    match e {
        EnvType::CMD => value.to_string(),
        EnvType::BASH | EnvType::POWERSHELL => {
            if value.len() == 0 {
                String::new()
            }
            else {
                let words: &[&str] =  &value.split('%').collect::<Vec<&str>>();
                let mut ret: String = String::new();
                let mut var = true;
                for word in words {
                    var = !var;
                    if var {
                        if word.len() == 0 {
                            ret.push('%');
                        }
                        else {
                            let new_var = match e {
                                EnvType::BASH => format!("${{{}}}", word),
                                EnvType::CMD => panic!(),
                                EnvType::POWERSHELL => format!("${{env:{}}}", word)
                            };
                            ret.push_str(&new_var);
                        }
                    }
                    else {
                        ret.push_str(word);
                    }
                }
                if var && value.chars().last().unwrap() != '%' {
                    panic!("incorrect % string in:{}", value)
                }
                ret
            }
        }
    }
}

#[test]
fn test_transform_vars() {
    assert_eq!(transform_vars("", EnvType::BASH), "");
    assert_eq!(transform_vars("b", EnvType::BASH), "b");
    assert_eq!(transform_vars("%ASD%", EnvType::BASH), "${ASD}");
    assert_eq!(transform_vars("%ASD%b", EnvType::BASH), "${ASD}b");
    assert_eq!(transform_vars("a%ASD%b", EnvType::BASH), "a${ASD}b");
    assert_eq!(transform_vars("a%%ASDb", EnvType::BASH), "a%ASDb");
}

#[test]
#[should_panic]
fn test_transform_vars_fail() {
    transform_vars("a%b", EnvType::BASH);
}

#[test]
#[should_panic]
fn test_transform_vars_fail2() {
    transform_vars("a%%ASD%b", EnvType::BASH);
}

fn generate_mod_env_set_value(eval_value: &str, t: EnvType) -> String {
    match t {
        EnvType::CMD | EnvType::POWERSHELL => format!("{}", eval_value),
        EnvType::BASH => format!("'{}'", &eval_value),
    }
}

fn generate_mod_env_value(name: &str, value: &str, m: ModType, e: EnvType) -> String {
    let eval_value = transform_vars(value, e);
    match m {
        ModType::PREPEND_PATH => {
            let mut s = generate_fix_path(&eval_value,e);
            s.push_str(generate_separator(e));
            s.push_str(&generate_get_env(name, e));
            s
        },
        ModType::APPEND_PATH => {
            let mut s = generate_get_env(name, e);
            s.push_str(generate_separator(e));
            s.push_str(&generate_fix_path(&eval_value,e));
            s
        },
        ModType::SET => generate_mod_env_set_value(&eval_value,e),
        ModType::PATH => generate_fix_path(&eval_value,e),
    }
}

fn generate_mod_env(name: &str, value: &str, m: ModType, e: EnvType) -> String {
    let mod_env_val = generate_mod_env_value(name, value, m, e);
    match e {
        EnvType::CMD => format!("set {}={}\r\n", name, &mod_env_val),
        EnvType::POWERSHELL => format!("$env:{}=\"{}\"\r\n", name, &mod_env_val),
        EnvType::BASH => format!("export {}={}\n", name, &mod_env_val),
    }
}

fn generate_src_env(file_to_src: &Path, e: EnvType) -> String {
    match e {
        EnvType::CMD => format!("call %~dp0\\{}\r\n", file_to_src.display()),
        EnvType::BASH => format!("source {}\n", file_to_src.display()),
        EnvType::POWERSHELL => format!(". {}\r\n", file_to_src.display()),
    }
}

#[derive(Deserialize)]
struct Config  {
    scripts: HashMap<String, Vec<std::collections::HashMap<String, String>>>,
}

#[derive(Deserialize)]
struct Args {
    flag_config: String,
    flag_output: String,
}

fn get_script_output_path(e: EnvType, out_path_str: &str, script_name: &str) -> PathBuf {
    let (subdir, extension) = match e {
        EnvType::CMD => ("cmd", "bat"),
        EnvType::POWERSHELL => ("ps", "ps1"),
        EnvType::BASH => ("bash", "sh")
    };
    let mut fname : String= "env_".to_string();
    fname.push_str(script_name);
    Path::new(out_path_str).join(subdir).join(&fname).with_extension(extension)
}

fn get_mod_type_by_str(s: &str) -> ModType {
    match s {
        "PREPEND_PATH" => ModType::PREPEND_PATH,
        "APPEND_PATH" => ModType::APPEND_PATH,
        "SET" => ModType::SET,
        "PATH" => ModType::PATH,
        _ => panic!("invalid mod type:{}", s)
    }
}

fn generate_script(script_name_pair: &(String, Vec<std::collections::HashMap<String, String>>), out_path_str: &str, e: EnvType) {
    let script_name = &script_name_pair.0;
    let cmds = &script_name_pair.1;
    let out_path = get_script_output_path(e, out_path_str, &script_name);
    if let Err(why) = std::fs::create_dir_all(out_path.parent().unwrap())  {
         panic!("couldn't create dir {}: {}", out_path.parent().unwrap().display(),
                                                   Error::description(&why))
    }
    
    let mut file = match File::create(&out_path) {
        Err(why) => panic!("couldn't create {}: {}",
                           out_path.display(),
                           Error::description(&why)),
        Ok(file) => file,
    };
    
    let mut out_content = String::new();
    match e {
        EnvType::CMD => {
            out_content.push_str(&format!("@rem {}\r\n", AUTOREMOVE_MARKER));
        },
        EnvType::BASH => {
            out_content.push_str(&format!("# {}\n", AUTOREMOVE_MARKER));
        }
        EnvType::POWERSHELL => {
            out_content.push_str(&format!("# {}\r\n", AUTOREMOVE_MARKER));
        }
    }
    for command in cmds {
        match &command.get("command").unwrap()[..] { 
            "env" => {
                let key = command.get("key").unwrap();
                let value = command.get("value").unwrap();
                let mode : ModType = get_mod_type_by_str(&command.get("mode").unwrap());
                out_content.push_str(&generate_mod_env(key, value, mode,e))
            },
            "source" => {
                let env = command.get("env").unwrap();
                let env_name = get_script_output_path(e, out_path_str, env);
                let file_to_source = Path::new(env_name.file_name().unwrap());
                out_content.push_str(&generate_src_env(file_to_source, e));
            }
            c @ _ => panic!("invalid command type: {}", c)
        }
    }

    if let Err(why) = file.write_all(&out_content[..].as_bytes())  {
         panic!("couldn't write {}: {}", out_path.display(),
                                                   Error::description(&why))
    }
}

fn remove_old_scripts(dir: &str) {
    for &subdir in ["cmd", "ps", "bash"].iter() {
        let p = &[&dir, subdir].iter().collect::<PathBuf>();
        if !Path::exists(p) {
            continue;
        }
        for entry in std::fs::read_dir(p).unwrap() {
            let entry = entry.unwrap();
            let path = &entry.path();
            if path.is_file() && path.file_name().unwrap().to_string_lossy().starts_with("env_") {
                let mut line = String::new();
                {
                    let f = File::open(path).unwrap();
                    let mut f = std::io::BufReader::new(f);
                    f.read_line(&mut line).unwrap();
                }

                if line.contains(AUTOREMOVE_MARKER) {
                    std::fs::remove_file(path).unwrap();
                }
            }
        }
    }
}


fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    let config_path = Path::new(&args.flag_config);

    let mut file = match File::open(&config_path) {
        Err(why) => panic!("couldn't open {}: {}", config_path.display(),
                                                   Error::description(&why)),
        Ok(file) => file,
    };
    
    let mut config_string = String::new();
    if let Err(why) = file.read_to_string(&mut config_string)  {
         panic!("couldn't read {}: {}", config_path.display(),
                                                   Error::description(&why))
    }

    remove_old_scripts(&args.flag_output[..]);

    let data: Config = toml::from_str(&config_string).unwrap();
    for script in data.scripts {
        for env in &[EnvType::CMD, EnvType::BASH, EnvType::POWERSHELL] {
            generate_script(&script, &args.flag_output[..], *env);
        }
    }
}
