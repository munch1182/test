use libcommon::{
    Builder, Getter,
    ext::{Command, CommandInExt, PrettyStringExt},
    newerr,
    prelude::{Result, trace, warn},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    ffi::OsStr,
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
    thread,
};

const PREFOX_ANDRODID_STUDIO: &str = "studio-";

/**
 * 参考androidstudio的实现：https://cs.android.com/android-studio/platform/tools/adt/idea/+/mirror-goog-studio-master-dev:android-adb/src/com/android/tools/idea/adb/wireless/WiFiPairingServiceImpl.kt
 */
pub fn connect_with_pair(pair_info: PairInfo) -> ConnectResult {
    connect_judge(Some(pair_info))
}

pub fn connect() -> ConnectResult {
    connect_judge(None)
}

pub fn disconnect() -> ConnectResult {
    if execute_adb(["disconnect"]).is_ok() {
        ConnectResult::ConnectSuccess
    } else {
        ConnectResult::Err("disconnect fail".to_string())
    }
}

pub fn list() -> ConnectResult {
    if let Ok(s) = execute_adb(["mdns", "services"]) {
        ConnectResult::List(s)
    } else {
        ConnectResult::Err("disconnect fail".to_string())
    }
}

fn connect_judge(pair: Option<PairInfo>) -> ConnectResult {
    if !check() {
        return ConnectResult::Err("ckech adb error".to_string());
    }
    match scan() {
        Ok(mut s) => {
            if s.is_empty() {
                return ConnectResult::Err("no device found".to_string());
            }
            s.sort();

            let paring = s.iter().find(|s| s.server_type == ServerType::PairingCode);
            if let Some(paring) = paring
                && let Some(pair) = pair
            {
                return pair_impl(paring, pair);
            }
            let connect = s.iter().find(|s| s.server_type == ServerType::QrCode);
            if let Some(connect) = connect {
                return connect_impl(connect);
            }

            ConnectResult::Err("not found pair or connect".to_string())
        }
        Err(e) => {
            warn!("scan failed: {e}");
            ConnectResult::Err(format!("error: {e}"))
        }
    }
}

fn connect_impl(connect: &ScanItem) -> ConnectResult {
    match execute_adb(["connect", &connect.addr_str()]) {
        Ok(s) => {
            if s.starts_with("connected to") {
                return ConnectResult::ConnectSuccess;
            }
            ConnectResult::Err(format!("connect failed: {s}"))
        }
        Err(e) => ConnectResult::Err(format!("error: {e}")),
    }
}

fn pair_impl(pairing: &ScanItem, pair: PairInfo) -> ConnectResult {
    match execute_adb(["pair", &pairing.addr_str(), &pair.pwd]) {
        Ok(s) => {
            if s.starts_with("Successfully paired to") {
                thread::sleep(std::time::Duration::from_secs(5));
                return connect();
            }
            ConnectResult::Err(format!("pair failed:{s}"))
        }
        Err(e) => ConnectResult::Err(format!("error: {e}")),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ConnectResult {
    ConnectSuccess,
    Err(String),
    List(String),
}

fn scan() -> Result<Vec<ScanItem>> {
    execute_adb(["mdns", "services"])
        .map(|r| r.lines().skip(1).flat_map(ScanItem::try_from).collect())
}

#[derive(Debug, Builder, Getter, Deserialize, Serialize)]
pub struct PairInfo {
    server: String,
    pwd: String,
}

impl Default for PairInfo {
    fn default() -> Self {
        PairInfo::new()
    }
}

impl PairInfo {
    pub fn new() -> Self {
        PairInfo {
            server: format!("{}{}", PREFOX_ANDRODID_STUDIO, Self::len_str(10)),
            pwd: Self::len_str(12),
        }
    }

    pub fn from(server: impl Into<String>, pwd: impl Into<String>) -> Self {
        PairInfo {
            server: server.into(),
            pwd: pwd.into(),
        }
    }

    ///
    /// 生成adb wifi需要的配对格式
    pub fn generate(&self) -> String {
        format!("WIFI:T:ADB;S:{};P:{};;", self.server, self.pwd)
    }

    fn len_str(len: usize) -> String {
        let char_set: Vec<char> =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-+*/<>{}"
                .chars()
                .collect();
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            s.push(char_set[rand::random_range(0..char_set.len())]);
        }
        s
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ScanItem {
    pub name: String,
    pub addr: SocketAddrV4,
    pub server_type: ServerType,
}

// 只实现 PartialOrd 和 Ord
impl PartialOrd for ScanItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScanItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // 按 name 长度降序
        other
            .name
            .len()
            .cmp(&self.name.len())
            // 长度相同时，可以按其他字段排序
            .then_with(|| self.name.cmp(&other.name))
            .then_with(|| self.addr.cmp(&other.addr))
            .then_with(|| self.server_type.cmp(&other.server_type))
    }
}

impl ScanItem {
    fn addr_str(&self) -> String {
        self.addr.to_string()
    }
}

impl TryFrom<&str> for ScanItem {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let result = find(value).ok_or("find failed".to_string())?;

        trace!("result: {:?}", result);

        if result.len() != 4 {
            return Err("Invalid scan item".to_string());
        }

        let name = result[0].to_string();
        let ip =
            Ipv4Addr::from_str(&result[2]).map_err(|_| format!("Invalid ip: {}", result[2]))?;
        let port = result[3]
            .parse::<u16>()
            .map_err(|_| format!("Invalid port: {}", result[3]))?;

        let addr = SocketAddrV4::new(ip, port);
        let server_type = ServerType::from(name.as_str());
        trace!("name: {}, addr: {}, port: {}", name, addr, port);

        Ok(ScanItem {
            name,
            addr,
            server_type,
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ServerType {
    QrCode,
    PairingCode,
}

impl From<&str> for ServerType {
    fn from(value: &str) -> Self {
        if value.starts_with(PREFOX_ANDRODID_STUDIO) {
            Self::PairingCode
        } else {
            Self::QrCode
        }
    }
}

pub fn check() -> bool {
    match execute_adb(["mdns", "check"]) {
        Ok(s) => s.contains("mdns daemon version"),
        Err(_) => false,
    }
}

fn execute_adb<I, S>(cmd: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    execute("adb", cmd)
}

fn find(s: impl AsRef<str>) -> Option<Vec<String>> {
    let re = r#"([^\t]+)\s*(_adb-tls-(?:connect|pairing)\._tcp)\s*([^:]+):([0-9]+)"#;
    let regex = Regex::new(re).ok()?;
    let cap = regex.captures(s.as_ref())?;
    Some(
        cap.iter()
            .skip(1)
            .filter_map(|m| m.map(|x| x.as_str().trim().to_string()))
            .collect(),
    )
}

// 执行cmd命令，并返回结果
pub(crate) fn execute<I, S>(program: impl AsRef<str>, cmd: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::with_args(program.as_ref(), cmd);
    trace!("execute: {}", command.to_string_pretty());
    command
        .output()
        .map(|e| e.out_or_err())
        .map(|s| String::from_utf8_lossy(&s).to_string())
        .map_err(|e| newerr!(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::{logsetup, prelude::info};

    #[logsetup]
    #[test]
    fn test_regex() {
        fn test(s: &str) {
            let found = find(s);
            info!("find: {:?}", found);
            assert!(found.is_some());
            assert!(found.unwrap().len() == 4);
        }
        test("adb-orvgxk4dqw9dduin-nRR0TU     _adb-tls-connect._tcp   192.168.2.188:36891");
        test("adb-orvgxk4dqw9dduin-nRR0TU (2)    _adb-tls-connect._tcp   192.168.2.188:36891");
        test("studio-abcdefghij       _adb-tls-pairing._tcp   192.168.2.188:39467");
    }

    #[logsetup]
    #[test]
    fn test_scanitem() {
        fn test(s: &str) {
            let found = ScanItem::try_from(s);
            info!("found: {:?}", found);
            assert!(found.is_ok());
        }
        test("adb-orvgxk4dqw9dduin-nRR0TU     _adb-tls-connect._tcp   192.168.2.188:36891");
        test("studio-abcdefghij       _adb-tls-pairing._tcp   192.168.2.188:39467");
    }

    #[logsetup("trace")]
    #[test]
    fn test_scan() {
        let result = scan();
        info!("scan: {:?}", result);
    }

    #[logsetup]
    #[test]
    fn test_execute() {
        let tests = vec![
            vec!["version"],
            vec!["mdns", "check"],
            vec!["mdns", "services"],
        ];
        for test in tests {
            info!("adb {}", test.join(" "));
            let result = execute_adb(&test);
            assert!(result.is_ok(), "Failed to execute command: {:?}", result);
            info!("{}", result.unwrap());
        }
    }
}
