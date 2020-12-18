/*++ @file

  Copyright ©2020 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use std::fs;
use std::io::prelude::*;
use std::path;
use structopt::StructOpt;
use sqlite;

#[derive(StructOpt, Debug)]
/// Hosts management tool and ipmitool wrapper
struct Opts {
    #[structopt(subcommand)]
    cmd: Option<Command>,

    /// Override inteface <lanplus>
    #[structopt(short = "I")]
    interface: Option<String>,
    /// Override default if set on host IP
    #[structopt(short = "H")]
    ip: Option<String>,
    /// Override default if set on host user name
    #[structopt(short = "U")]
    user: Option<String>,
    /// Override default if set on host user password
    #[structopt(short = "P")]
    pswd: Option<String>,
    /// The ipmitool args to process
    ipmitool_args: Vec<String>,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Host management subcommand(s)
    Host {
        #[structopt(subcommand)]
        cmd: HostCommand,
    }
}

#[derive(StructOpt, Debug)]
enum HostCommand {
    /// List all IPMI hosts
    List,
    /// Add an IPMI host record
    Add {
        #[structopt(flatten)]
        host: Host,
    },
    /// Delete an IPMI host record
    Del {
        id: i64,
    },
    /// Set current IPMI host
    Use {
        id: i64,
    }
}

#[derive(StructOpt,Debug)]
struct Host {
    /// Host IP
    #[structopt(short, long)]
    ip: String,
    /// Host user name
    #[structopt(short, long)]
    user: String,
    /// Host user password
    #[structopt(short, long)]
    pswd: String,
}

impl Host {
    fn init(db: Option<&str>) -> (sqlite::Connection, path::PathBuf) {
        let datahome = if cfg!(target_os = "windows") { std::env::var("USERPROFILE").unwrap() } else { std::env::var("HOME").unwrap() };
        let database = path::Path::new(&datahome).join(db.unwrap_or(".ipmi.db"));
        let _1st_run = !database.is_file();

        let mut f = fs::OpenOptions::new().read(true).write(true).create(true).open(&database).unwrap();
        f.flush().unwrap();

        let connection = sqlite::open(&database).unwrap();
        if _1st_run {
            connection.execute(
                "
                CREATE TABLE hosts (id INTEGER PRIMARY KEY AUTOINCREMENT, df TINYINT DEFAULT 0, ip VARCHAR(64) NOT NULL, user VARCHAR(64) NOT NULL, pswd VARCHAR(64) NOT NULL);
                CREATE UNIQUE INDEX hi ON hosts (ip, user);
                "
            ).unwrap();
        }
        (connection, database)
    }
    fn list(connection: &sqlite::Connection) -> bool {
        let mut list_some = false;
        let mut statement = connection.prepare("SELECT id, df, ip, user FROM hosts ORDER BY ROWID ASC").unwrap();
        println!("---------------------------------------------");
        println!("Index        IP                          User");
        println!("-----        --                          ----");
        while let sqlite::State::Row = statement.next().unwrap() {
            let id   = statement.read::<i64>(0).unwrap();
            let df   = statement.read::<i64>(1).unwrap();
            let ip   = statement.read::<String>(2).unwrap();
            let user = statement.read::<String>(3).unwrap();

            let df_mark = if df != 0  { '*' } else { ' ' };
            println!("{}{:>04}        {:<15}  {:>15}", df_mark, id, ip, user);

            list_some = true;
        }
        println!("---------------------------------------------");

        list_some
    }
    fn set(connection: &sqlite::Connection, id: i64) -> bool {
        let mut id_ok = false;
        let mut statement = connection.prepare(format!("SELECT id FROM hosts WHERE id={}", id)).unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            id_ok = true;
        }

        if id == 0 {
            id_ok = true;
        }

        if id_ok {
            let mut statement = connection.prepare(format!("SELECT id FROM hosts WHERE df=1")).unwrap();
            while let sqlite::State::Row = statement.next().unwrap() {
                connection.execute(format!("UPDATE hosts SET df=0 WHERE id={}", statement.read::<i64>(0).unwrap())).unwrap();
            }
            if id != 0 {
                connection.execute(format!("UPDATE hosts SET df=1 WHERE id={}", id)).unwrap();
            }
        }

        id_ok
    }
    fn get(connection: &sqlite::Connection) -> Option<Host> {
        let mut host: Option<Host> = None;

        let mut statement = connection.prepare(format!("SELECT ip, user, pswd FROM hosts WHERE df=1")).unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            let ip   = statement.read::<String>(0).unwrap();
            let user = statement.read::<String>(1).unwrap();
            let pswd = statement.read::<String>(2).unwrap();
            host = Some(Host {ip, user, pswd});
        }
        host
    }
    fn add(connection: &sqlite::Connection, host: &Host) {
        if host.ip != "UNKNOWN" && host.user != "UNKNOWN" && host.pswd != "UNKNOWN" {
            let mut ip_ok = false;
            if let Ok(_) = host.ip.parse::<std::net::Ipv4Addr>() {
                ip_ok = true;
            }
            if let Ok(_) = host.ip.parse::<std::net::Ipv6Addr>() {
                ip_ok = true;
            }
            if !ip_ok {
                println!("Invalid IP: {}", host.ip);
                return;
            }

            connection.execute(
                format!(
                    "INSERT INTO hosts(ip, user, pswd) VALUES ('{}', '{}', '{}') ON CONFLICT(ip, user) DO UPDATE SET pswd=excluded.pswd;"
                , host.ip, host.user, host.pswd)
            ).unwrap();
            let mut _id_ = 0;
            let mut statement = connection.prepare("SELECT id FROM hosts ORDER BY ROWID ASC").unwrap();
            while let sqlite::State::Row = statement.next().unwrap() {
                _id_ += 1;
                let id = statement.read::<i64>(0).unwrap();
                if _id_ < id {
                    connection.execute(format!("UPDATE hosts SET id= {} WHERE id={}", _id_, id)).unwrap();
                }
            }
            connection.execute(format!("UPDATE sqlite_sequence SET seq={} WHERE name='hosts'", _id_)).unwrap();
        }
    }
    fn del(connection: &sqlite::Connection, id: i64) {
        connection.execute(format!("DELETE FROM hosts WHERE id={}", id)).unwrap();
        let mut _id_ = 0;
        let mut statement = connection.prepare("SELECT id FROM hosts ORDER BY ROWID ASC").unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            _id_ += 1;
            let id = statement.read::<i64>(0).unwrap();
            if _id_ < id {
                connection.execute(format!("UPDATE hosts SET id= {} WHERE id={}", _id_, id)).unwrap();
            }
        }
        connection.execute(format!("UPDATE sqlite_sequence SET seq={} WHERE name='hosts'", _id_)).unwrap();
    }
    fn with_args(&self, opt: &Opts) -> String {
        let mut ipmitool_args = format!("ipmitool -I {} -H {} -U {} -P {}",
            opt.interface.as_ref().unwrap_or(&String::from("lanplus")), opt.ip.as_ref().unwrap_or(&self.ip), opt.user.as_ref().unwrap_or(&self.user), opt.pswd.as_ref().unwrap_or(&self.pswd)
        );

        let mut ipmitool_rest = String::new();
        for i in &opt.ipmitool_args {
            ipmitool_rest.push(' ');
            ipmitool_rest.push_str(&i);
        }
        ipmitool_args.push_str(&ipmitool_rest);

        ipmitool_args
    }
}

fn main() {
    let opt = Opts::from_args();

    let (connection, _) = Host::init(None);

    if let Some(Command::Host{cmd}) = opt.cmd {
        match cmd {
            HostCommand::List => {
                println!();
                if !Host::list(&connection) {
                    println!("Please add at least one host:");
                    println!("    ipmi.exe host add -i <ip> -u <user> -p <pswd>");
                    println!("And then use it:");
                    println!("    ipmi.exe host use <index of host>");
                }
                println!();
            },
            HostCommand::Use{id} => {
                if !Host::set(&connection, id) {
                    println!("Please list and find available <index of host>:");
                    println!("    ipmi.exe host list");
                }
            },
            HostCommand::Add{host} => {
                Host::add(&connection, &host);
            },
            HostCommand::Del{id} => {
                Host::del(&connection, id);
            }
        }
    } else {
        if let Some(host) = Host::get(&connection) {
            use std::process::Command;

            let cmd: (&str, &str) = if cfg!(target_os = "windows") { ("cmd", "/c") } else { ("sh", "-c") };
            if let Ok(output) = Command::new(cmd.0).arg(cmd.1).arg(host.with_args(&opt)).output() {
                if output.status.success() {
                    std::io::stdout().write_all(&output.stdout).unwrap();
                } else {
                    std::io::stdout().write_all(&output.stderr).unwrap();
                }
            } else {
                println!("Unknown error when run: {}", host.with_args(&opt));
            }
        } else {
            println!("Please set default host with command:");
            println!("    ipmi.exe host use <index of host>");
        }
    }
}

#[test]
fn host_init() {
    let (connection, database) = Host::init(None);
    //.1 check database file
    assert!(database.is_file());
    //.2 check database table
    if let Ok(_) = connection.execute("SELECT ip, user, pswd FROM hosts") {
        assert!(true);
    } else {
        assert!(false);
    }
}

#[test]
fn host_list() {
    let db_name = "list.db";

    let database = {
        let (_, database) = Host::init(Some(db_name));
        database.clone()
    };
    fs::remove_file(&database).unwrap_or(());

    { // case: some in list or none in list
        let (connection, _) = Host::init(Some(db_name));

        assert_eq!(Host::list(&connection), false);

        let d0 = (1i64, 0i64, String::from("000.000.000.000"), String::from("admin"), String::from("admin"));
        connection.execute(
            format!("INSERT INTO hosts VALUES ({}, {}, '{}', '{}', '{}')", d0.0, d0.1, d0.2, d0.3, d0.4)
        ).unwrap();
        assert_eq!(Host::list(&connection), true);
    }
    fs::remove_file(&database).unwrap();
}

#[test]
fn host_add() {
    let db_name = "add.db";

    let database = {
        let (_, database) = Host::init(Some(db_name));
        database.clone()
    };
    fs::remove_file(&database).unwrap_or(());

    let d0 = (1i64, 0i64, String::from("000.000.000.000"), String::from("admin"), String::from("admin"));
    let d1 = (2i64, 0i64, String::from("255.255.255.255"), String::from("ADMIN"), String::from("ADMIN"));
    let _t : (i64, i64, String, String, String);
    { // case: original equal 1
        let (connection, _) = Host::init(Some(db_name));
        connection.execute(
            format!("INSERT INTO hosts VALUES ({}, {}, '{}', '{}', '{}')", d0.0, d0.1, d0.2, d0.3, d0.4)
        ).unwrap();
        let mut statement = connection.prepare("SELECT * FROM hosts ORDER BY ROWID ASC").unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            let t0: (i64, i64, String, String, String) = (
                statement.read::<i64>(0).unwrap(),
                statement.read::<i64>(1).unwrap(),
                statement.read::<String>(2).unwrap(),
                statement.read::<String>(3).unwrap(),
                statement.read::<String>(4).unwrap(),
            );
            assert_eq!(t0, d0);
        }
    }
    fs::remove_file(&database).unwrap();
    { // case: original equal 2
        let (connection, _) = Host::init(Some(db_name));
        let host = Host { ip: String::from(&d0.2), user: String::from(&d0.3), pswd: String::from(&d0.4) };
        Host::add(&connection, &host);
        let mut statement = connection.prepare("SELECT * FROM hosts ORDER BY ROWID ASC").unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            let t0: (i64, i64, String, String, String) = (
                statement.read::<i64>(0).unwrap(),
                statement.read::<i64>(1).unwrap(),
                statement.read::<String>(2).unwrap(),
                statement.read::<String>(3).unwrap(),
                statement.read::<String>(4).unwrap(),
            );
            assert_eq!(t0, d0);
        }
    }
    fs::remove_file(&database).unwrap();

    { // case: edge value
        let (connection, _) = Host::init(Some(db_name));
        let host1 = Host { ip: String::from(&d0.2), user: String::from(&d0.3), pswd: String::from(&d0.4) };
        let host2 = Host { ip: String::from(&d1.2), user: String::from(&d1.3), pswd: String::from(&d1.4) };
        Host::add(&connection, &host1);
        Host::add(&connection, &host2);
        let mut statement = connection.prepare("SELECT ip, user, pswd FROM hosts ORDER BY ROWID ASC").unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            let ip   = statement.read::<String>(0).unwrap();
            let user = statement.read::<String>(1).unwrap();
            let pswd = statement.read::<String>(2).unwrap();

            if ip == host1.ip && user == host1.user {
                assert_eq!(pswd, host1.pswd);
            } else {
                assert_eq!(pswd, host2.pswd);
            }
        }
    }
    fs::remove_file(&database).unwrap();

    let d0 = (1i64, 0i64, String::from("200.050.005.000"), String::from("ADMIN"), String::from("ADMIN"));
    let d1 = (2i64, 0i64, String::from("200.050.005.000"), String::from("ADmin"), String::from("ADMIN"));
    let d2 = (3i64, 0i64, String::from("200.050.005.000"), String::from("ADmin"), String::from("adMIN"));
    let d3 = (4i64, 0i64, String::from("200.050.005.001"), String::from("ADmin"), String::from("adMIN"));
    { // case: unique on (ip, user) 1
        let (connection, _) = Host::init(Some(db_name));
        let host1 = Host { ip: String::from(&d0.2), user: String::from(&d0.3), pswd: String::from(&d0.4) };
        let host2 = Host { ip: String::from(&d1.2), user: String::from(&d1.3), pswd: String::from(&d1.4) };
        Host::add(&connection, &host1);
        Host::add(&connection, &host2);
        let mut statement = connection.prepare("SELECT ip, user, pswd FROM hosts ORDER BY ROWID ASC").unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            let ip   = statement.read::<String>(0).unwrap();
            let user = statement.read::<String>(1).unwrap();
            let pswd = statement.read::<String>(2).unwrap();

            if ip == host1.ip && user == host1.user {
                assert_eq!(pswd, host1.pswd);
            } else {
                assert_eq!(pswd, host2.pswd);
            }
        }
    }
    fs::remove_file(&database).unwrap();
    { // case: unique on (ip, user) 2
        let (connection, _) = Host::init(Some(db_name));
        let host1 = Host { ip: String::from(&d1.2), user: String::from(&d1.3), pswd: String::from(&d1.4) };
        let host2 = Host { ip: String::from(&d2.2), user: String::from(&d2.3), pswd: String::from(&d2.4) };
        Host::add(&connection, &host1);
        Host::add(&connection, &host2);
        let mut i = 0;
        let mut statement = connection.prepare("SELECT ip, user, pswd FROM hosts ORDER BY ROWID ASC").unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            let ip   = statement.read::<String>(0).unwrap();
            let user = statement.read::<String>(1).unwrap();
            let pswd = statement.read::<String>(2).unwrap();

            assert_eq!(ip,   host1.ip);
            assert_eq!(user, host1.user);
            assert_ne!(pswd, host1.pswd);
            assert_eq!(pswd, host2.pswd);

            i += 1;
        }
        assert_eq!(i, 1);
    }
    fs::remove_file(&database).unwrap();
    { // case: successive order
        let (connection, _) = Host::init(Some(db_name));
        let host1 = Host { ip: String::from(&d0.2), user: String::from(&d0.3), pswd: String::from(&d0.4) };
        let host2 = Host { ip: String::from(&d1.2), user: String::from(&d1.3), pswd: String::from(&d1.4) };
        let host3 = Host { ip: String::from(&d2.2), user: String::from(&d2.3), pswd: String::from(&d2.4) };
        let host4 = Host { ip: String::from(&d3.2), user: String::from(&d3.3), pswd: String::from(&d3.4) };
        Host::add(&connection, &host1);
        Host::add(&connection, &host2);
        Host::add(&connection, &host3);
        Host::add(&connection, &host4);
        let mut _id_ = 0;
        let mut statement = connection.prepare("SELECT id FROM hosts ORDER BY ROWID ASC").unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            let id = statement.read::<i64>(0).unwrap();

            _id_ += 1;
            assert_eq!(id, _id_);
        }
    }
    fs::remove_file(&database).unwrap();
}

#[test]
fn host_del() {
    let db_name = "del.db";

    let database = {
        let (_, database) = Host::init(Some(db_name));
        database.clone()
    };
    fs::remove_file(&database).unwrap_or(());

    let d0 = (1i64, 0i64, String::from("000.000.000.000"), String::from("admin"), String::from("admin"));
    let d1 = (2i64, 0i64, String::from("255.255.255.255"), String::from("ADMIN"), String::from("ADMIN"));
    let d2 = (3i64, 0i64, String::from("200.050.005.000"), String::from("ADMIN"), String::from("ADMIN"));
    let d3 = (4i64, 0i64, String::from("200.050.005.000"), String::from("ADmin"), String::from("ADMIN"));
    let vd : [(i64, i64, String, String, String);4] = [ d0.clone(), d1.clone(), d2.clone(), d3.clone() ];
    { // case: delete and reorder
        let (connection, _) = Host::init(Some(db_name));
        for i in &vd {
            connection.execute(
                format!("INSERT INTO hosts VALUES ({}, {}, '{}', '{}', '{}')", i.0, i.1, i.2, i.3, i.4)
            ).unwrap();
        }
        for i in &vd {
            Host::del(&connection, 1);

            let mut id = 0;
            let mut statement = connection.prepare("SELECT * FROM hosts ORDER BY ROWID ASC").unwrap();
            while let sqlite::State::Row = statement.next().unwrap() {
                let t0: (i64, String, String, String) = (
                    statement.read::<i64>(1).unwrap(),
                    statement.read::<String>(2).unwrap(),
                    statement.read::<String>(3).unwrap(),
                    statement.read::<String>(4).unwrap(),
                );
                let d0: (i64, String, String, String) = (i.1.clone(), i.2.clone(), i.3.clone(), i.4.clone());
                // case: no this record after delete
                assert_ne!(t0, d0);

                id += 1;
                // case: successive order after delete
                assert_eq!(id, statement.read::<i64>(0).unwrap());
            }
        }
    }
    fs::remove_file(&database).unwrap();
}

#[test]
fn host_set() {
    let db_name = "set.db";

    let database = {
        let (_, database) = Host::init(Some(db_name));
        database.clone()
    };
    fs::remove_file(&database).unwrap_or(());

    let d0 = (1i64, 0i64, String::from("000.000.000.000"), String::from("admin"), String::from("admin"));
    let d1 = (2i64, 0i64, String::from("255.255.255.255"), String::from("ADMIN"), String::from("ADMIN"));
    let d2 = (3i64, 0i64, String::from("200.050.005.000"), String::from("ADMIN"), String::from("ADMIN"));
    let d3 = (4i64, 0i64, String::from("200.050.005.000"), String::from("ADmin"), String::from("ADMIN"));
    let vd : [(i64, i64, String, String, String);4] = [ d0.clone(), d1.clone(), d2.clone(), d3.clone() ];
    { // case: default can be set and unique
        let (connection, _) = Host::init(Some(db_name));
        for i in &vd {
            connection.execute(
                format!("INSERT INTO hosts VALUES ({}, {}, '{}', '{}', '{}')", i.0, i.1, i.2, i.3, i.4)
            ).unwrap();
        }

        let mut id:i64 = 1;
        for (i, v) in vd.iter().enumerate() {
            assert_eq!(id, (i+1) as i64);
            Host::set(&connection, id);

            let mut n = 0;
            let mut statement = connection.prepare("SELECT * FROM hosts WHERE df=1").unwrap();
            while let sqlite::State::Row = statement.next().unwrap() {
                let t0: (i64, String, String, String) = (
                    statement.read::<i64>(0).unwrap(),
                    statement.read::<String>(2).unwrap(),
                    statement.read::<String>(3).unwrap(),
                    statement.read::<String>(4).unwrap(),
                );
                assert_eq!(statement.read::<i64>(1).unwrap(), 1);
                let d0: (i64, String, String, String) = (v.0.clone(), v.2.clone(), v.3.clone(), v.4.clone());
                assert_eq!(t0, d0);
                n += 1;
            }
            assert_eq!(n, 1);
            id += 1;
        }

        // case: id is valid or not
        assert_eq!(Host::set(&connection, d3.0 + 1), false);

        // case: id = 0 is valid for clear default
        assert_eq!(Host::set(&connection, 0), true);
        let mut statement = connection.prepare(format!("SELECT id FROM hosts WHERE df=1")).unwrap();
        while let sqlite::State::Row = statement.next().unwrap() {
            assert!(false);
        }
    }
    fs::remove_file(&database).unwrap();
}

#[test]
fn host_get() {
    let db_name = "get.db";

    let database = {
        let (_, database) = Host::init(Some(db_name));
        database.clone()
    };
    fs::remove_file(&database).unwrap_or(());

    let d0 = (1i64, 0i64, String::from("000.000.000.000"), String::from("admin"), String::from("admin"));
    let d1 = (2i64, 0i64, String::from("255.255.255.255"), String::from("ADMIN"), String::from("ADMIN"));
    let d2 = (3i64, 0i64, String::from("200.050.005.000"), String::from("ADMIN"), String::from("ADMIN"));
    let d3 = (4i64, 0i64, String::from("200.050.005.000"), String::from("ADmin"), String::from("ADMIN"));
    let vd : [(i64, i64, String, String, String);4] = [ d0.clone(), d1.clone(), d2.clone(), d3.clone() ];
    { // case: default can be got
        let (connection, _) = Host::init(Some(db_name));
        for i in &vd {
            connection.execute(
                format!("INSERT INTO hosts VALUES ({}, {}, '{}', '{}', '{}')", i.0, i.1, i.2, i.3, i.4)
            ).unwrap();
        }

        let mut id:i64 = 1;
        for (i, v) in vd.iter().enumerate() {
            assert_eq!(id, (i+1) as i64);
            connection.execute(format!("UPDATE hosts SET df=1 WHERE id={}", id)).unwrap();

            let host1 = Host { ip: String::from(&v.2), user: String::from(&v.3), pswd: String::from(&v.4) };

            if let Some(host2) = Host::get(&connection) {
                assert_eq!(host1.ip, host2.ip);
                assert_eq!(host1.user, host2.user);
                assert_eq!(host1.pswd, host2.pswd);
            } else {
                assert!(false);
            }

            id += 1;
        }
    }
    fs::remove_file(&database).unwrap();
}

#[test]
fn host_with_args() {
    let host = Host { ip: String::from("000.000.000.000"), user: String::from("admin"), pswd: String::from("admin") };

    // case: use database default host
    let opts = Opts { cmd: None, interface: None, ip: None, user: None, pswd: None, ipmitool_args: Vec::new() };
    assert_eq!(host.with_args(&opts), "ipmitool -I lanplus -H 000.000.000.000 -U admin -P admin");

    // case: override database default
    let opts = Opts { cmd: None, interface: Some(String::from("lan")), ip: Some(String::from("200.050.005.000")), user: Some(String::from("ADMIN")), pswd: Some(String::from("ad*in")), ipmitool_args: Vec::new() };
    assert_eq!(host.with_args(&opts), "ipmitool -I lan -H 200.050.005.000 -U ADMIN -P ad*in");

    // case: with ipmitool_args
    let opts = Opts { cmd: None, interface: None, ip: None, user: None, pswd: None, ipmitool_args: vec![String::from("1st"), String::from("sec")] };
    assert_eq!(host.with_args(&opts), "ipmitool -I lanplus -H 000.000.000.000 -U admin -P admin 1st sec");
}
