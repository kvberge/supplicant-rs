use std::{error::Error, collections::HashMap};
use futures_util::StreamExt;
use supplicant::{Interface, Supplicant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let supplicant = Supplicant::connect().await?;

    let sup = supplicant.clone();

    let (tx,mut rx) = tokio::sync::mpsc::unbounded_channel::<zbus::zvariant::ObjectPath>();
    // listen for interface add/remove, happens when connman restarts for example
    tokio::spawn( async move {
        let mut iface_added = sup.iface_added().await;
        let mut iface_removed = sup.iface_removed().await;
        loop {
            tokio::select! {
                Some(iface) = iface_removed.next() => {
                    println!("Interface removed {:?}", iface);
                    if let Ok(b) = iface.body::<zbus::zvariant::ObjectPath>() {
                        println!("{:?}", b);
                    } else {
                        println!("Was error");
                    }
                }
                Some(iface) = iface_added.next() => {
                    println!("Interface added {:?}", iface);
                    if let Ok(b) = iface.body::<(zbus::zvariant::ObjectPath, HashMap<String, zbus::zvariant::Value>)>() {
                        println!("{:?}", b.0);
                        if let Some(name) = b.1.get("Ifname") {
                            let n: String = name.try_into().unwrap();
                            println!("Name: {:?}", n);
                        }
                    } else {
                        println!("Was error");
                    }
                }
            }
        }
    });

    let s = supplicant.clone();
    // let wlan_interface = find_interface(&s, "wlan0")
    //     .await
    //     .ok_or("Failed to find wlan0")?;
    println!("Awaiting signals");
    let j = tokio::spawn( async move {
        let mut iface = Interface::new(s.conn.clone(), &s.proxy, path.into()).await.expect("Didn't find interface");

        let mut auth = iface.auth_signal().await;
        loop {
            tokio::select! {
                Some(path) = rx.recv() => {
                    iface = Interface::new(s.conn.clone(), &s.proxy, path.into()).await.expect("Didn't find interface");

                    auth = iface.auth_signal().await;
                }
                Some(m) = auth.next() => {
                    println!("Got Auth signal: {:?}",m.get().await);
                    // let by = m.body_as_bytes().unwrap();
                    // let b: zbus::zvariant::Structure = m.body().unwrap();
                    // println!("Body is {:?}", b);
                    // println!("{:?}", String::from_utf8_lossy(&by));
                    // println!("{:?}", m.body_as_bytes());
                }
            }
        }
    });
    tokio::join!(j);
    Ok(())


}

async fn find_interface<'a>(
    supplicant: &'a Supplicant<'_>,
    iface_name: impl AsRef<str>,
) -> Option<Result<Interface<'a>, zbus::Error>> {
    let mut iface_res: Option<Result<_, zbus::Error>> = None;
    for iface in supplicant.interfaces().await.unwrap().into_iter() {
        let ifname = iface.ifname().await;
        match ifname {
            Ok(name) if &name == iface_name.as_ref() => {
                iface_res = Some(Ok(iface));
                break;
            }
            // Store the last err to return at the end
            Err(e) => {
                iface_res = Some(Err(e));
            }
            // Ignore other ifaces
            Ok(_) => {}
        }
    }
    iface_res
}
