use std::fmt::Debug;

use futures::Stream;
use kube::{
    api::{DynamicObject, ListParams, ObjectMeta, PartialObjectMeta, Request},
    Api, Client, Resource,
};
use kube_core::WatchEvent;
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Clone, Deserialize, Debug)]
struct ColumnDefinition {
    name: String,
    r#type: String,
    format: String,
    description: String,
    priority: isize,
}

#[derive(Clone, Deserialize, Debug)]
struct Row<K = DynamicObject> {
    cells: Vec<String>,
    object: PartialObjectMeta<K>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Table<K = DynamicObject> {
    metadata: ObjectMeta,
    column_definitions: Option<Vec<ColumnDefinition>>,
    rows: Vec<Row<K>>,
}

// impl<R: Resource> Resource for Table<R> {
//     const API_VERSION: &'static str = R::API_VERSION;
//     const GROUP: &'static str = R::GROUP;
//     const KIND: &'static str = R::KIND;
//     const VERSION: &'static str = R::VERSION;
//     const URL_PATH_SEGMENT: &'static str = R::URL_PATH_SEGMENT;

//     type Scope = R::Scope;
// }

// impl<R: Resource> k8s_openapi::Metadata for Table<R> {
//     type Ty = ObjectMeta;

//     fn metadata(&self) -> &Self::Ty {
//         &self.metadata
//     }

//     fn metadata_mut(&mut self) -> &mut Self::Ty {
//         &mut self.metadata
//     }
// }

pub struct DemoApi<K> {
    /// The request builder object with its resource dependent url
    pub request: kube::api::Request,
    /// The client to use (from this library)
    pub client: kube::Client,
    namespace: Option<String>,
    /// Note: Using `iter::Empty` over `PhantomData`, because we never actually keep any
    /// `K` objects, so `Empty` better models our constraints (in particular, `Empty<K>`
    /// is `Send`, even if `K` may not be).
    pub _phantom: std::iter::Empty<K>,
}

impl<K: Resource> DemoApi<K>
where
    K: Clone + DeserializeOwned + Debug,
    <K as Resource>::DynamicType: Default,
{
    pub fn all(client: Client) -> Self {
        Self::all_with(client, &K::DynamicType::default())
    }
}

impl<K: Resource> DemoApi<K>
where
    K: Clone + DeserializeOwned + Debug,
{
    pub fn all_with(client: Client, dyntype: &K::DynamicType) -> Self {
        let url = K::url_path(dyntype, None);
        Self {
            client,
            request: Request::new(url),
            namespace: None,
            _phantom: std::iter::empty(),
        }
    }

    async fn get_table(&self, name: &str) -> Result<Table<K>, kube::Error> {
        let target = format!("{}/{}?", self.request.url_path, name);

        let req: http::Request<Vec<u8>> = self
            .request(&target)
            .body(vec![])
            .map_err(kube_core::request::Error::BuildRequest)
            .unwrap();

        let res = self.client.request_text(req).await.unwrap();
        dbg!(&res);
        let table: Table<K> = serde_json::from_str(&res).unwrap();

        Ok(table)
    }

    async fn list_table(&self, lp: &ListParams) -> Result<Table<K>, kube::Error> {
        // let target = format!("{}/{}?", self.request.url_path, name);

        let req: http::Request<Vec<u8>> = self
            .request(&self.request.url_path)
            .body(vec![])
            .map_err(kube_core::request::Error::BuildRequest)
            .unwrap();

        let res = self.client.request_text(req).await.unwrap();
        dbg!(&res);
        let table: Table<K> = serde_json::from_str(&res).unwrap();

        Ok(table)
    }

    async fn watch(
        &self,
        version: &str,
    ) -> Result<impl Stream<Item = Result<WatchEvent<Table<K>>, kube::Error>>, kube::Error> {
        let target = format!("{}?", self.request.url_path);
        let mut qp = form_urlencoded::Serializer::new(target);
        qp.append_pair("watch", "true");
        qp.append_pair("sendInitialEvents", "true");
        qp.append_pair("resourceVersionMatch", "NotOlderThan");
        qp.append_pair("resourceVersion", version);
        let urlstr = qp.finish();
        let req = self.request(&urlstr);
        let req = req.body(vec![]).unwrap();

        self.client.request_events::<Table<K>>(req).await
    }

    fn request(&self, url: &str) -> http::request::Builder {
        http::Request::get(url).header("Accept", "application/json;as=Table;v=v1;g=meta.k8s.io")
    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;

    use k8s_openapi::api::core::v1::Namespace;
    use tokio_stream::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test() {
        let client = kube::Client::try_default().await.unwrap();
        let namespaces: DemoApi<Namespace> = DemoApi::all(client);

        let mut res = pin!(namespaces.watch("0").await.unwrap());

        while let Some(event) = res.next().await {
            match event {
                Ok(event) => match event {
                    WatchEvent::Added(r) => {
                        for row in r.rows {
                            print_table(&vec![row.cells]);
                        }
                    }
                    WatchEvent::Modified(_) => {}
                    WatchEvent::Deleted(_) => {}
                    WatchEvent::Bookmark(_) => {}
                    WatchEvent::Error(_) => {}
                },
                Err(e) => eprintln!("{e}"),
            }
        }
    }

    // #[tokio::test]
    // async fn test() {
    //     let client = kube::Client::try_default().await.unwrap();

    //     let namespaces: DemoApi<Namespace> = DemoApi::all(client);

    //     // let res = namespaces.get_table("default").await.unwrap();
    //     let res = namespaces.list_table(&ListParams::default()).await.unwrap();
    //     dbg!(&res);

    //     let header: Vec<String> = res
    //         .column_definitions
    //         .into_iter()
    //         .map(|c| c.name.to_uppercase())
    //         .collect();
    //     let rows = res.rows.into_iter().map(|r| r.cells);

    //     print_table(&std::iter::once(header).chain(rows).collect());
    // }

    fn print_table(table: &Vec<Vec<String>>) {
        if table.is_empty() {
            return;
        }

        // Number of columns
        let cols = table[0].len();

        // Compute max width for each column
        let mut widths = vec![0; cols];
        for row in table {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(cell.len() + 2);
            }
        }

        // Print each row with padding
        for row in table {
            for (i, cell) in row.iter().enumerate() {
                print!("{:width$} ", cell, width = widths[i]);
            }
            println!();
        }
    }
}
