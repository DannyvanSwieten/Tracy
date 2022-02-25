use super::application::Model;
use super::schema::new_schema;
use super::Renderer;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_poem::GraphQL;
use futures::channel::oneshot;
use futures::lock::Mutex;
use poem::{get, handler, listener::TcpListener, post, web::Html, IntoResponse, Route};
use std::{sync::Arc, thread, time::Duration};
use tokio::runtime;
pub struct Server {
    sender: Option<oneshot::Sender<()>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl Server {
    pub fn new<A>(model: Arc<Mutex<Model>>, addr: A, serve_playground: bool) -> Self
    where
        A: Into<String>,
    {
        let (sender, receiver) = oneshot::channel();

        let addr = addr.into();

        let join_handle = thread::spawn(move || {
            let runtime = runtime::Builder::new_current_thread()
                .thread_name("Server")
                .enable_all()
                .build()
                .expect("Could not build Tokio server");

            runtime
                .block_on(async {
                    let schema = new_schema(model);

                    let endpoint = if serve_playground {
                        println!("Serving playground at http://{addr}");
                        get(graphql_playground).post(GraphQL::new(schema))
                    } else {
                        post(GraphQL::new(schema))
                    };
                    let app = Route::new().at("/", endpoint);

                    poem::Server::new(TcpListener::bind(addr))
                        .run_with_graceful_shutdown(
                            app,
                            async move {
                                let _ = receiver.await;
                            },
                            Some(Duration::from_secs(1)),
                        )
                        .await
                })
                .unwrap();
        });

        Self {
            sender: Some(sender),
            join_handle: Some(join_handle),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.sender
            .take()
            .unwrap()
            .send(())
            .expect("Could not send shutdown for server thread");

        self.join_handle
            .take()
            .unwrap()
            .join()
            .expect("Could not join server thread");
    }
}

#[handler]
async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}
