// Copyright 2022 Quentin Gliech
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use bytes::{Buf, Bytes};
use http::{Request, Response};
use http_body::{Body, Full};
use pyo3::{exceptions::PyValueError, types::PyType, FromPyObject, PyAny, PyErr, PyResult};
use pyo3_twisted_web::Resource;
use serde::Deserialize;
use tower_service::Service;

mod synapse {
    pyo3::import_exception!(synapse.module_api.errors, ConfigError);
}

pub struct ModuleApi<'a> {
    inner: &'a PyAny,
}

impl<'a> ModuleApi<'a> {
    /// Register a [`Service`] to handle a path
    ///
    /// # Errors
    ///
    /// Returns an error if the call to `ModuleApi.register_web_resource` failed
    pub fn register_web_service<S, B, E>(&self, path: &str, service: S) -> PyResult<()>
    where
        S: Service<Request<Full<Bytes>>, Response = Response<B>, Error = E>
            + Clone
            + Send
            + 'static,
        S::Future: Send,
        B: Body + Send + 'static,
        B::Data: Buf + 'static,
        B::Error: Into<PyErr> + 'static,
        E: Into<PyErr> + 'static,
    {
        self.inner.call_method1(
            "register_web_resource",
            (path, Resource::from_service(service)),
        )?;
        Ok(())
    }
}

impl<'a> FromPyObject<'a> for ModuleApi<'a> {
    fn extract(inner: &'a PyAny) -> PyResult<Self> {
        let module_api_cls = inner
            .py()
            .import("synapse.module_api")?
            .getattr("ModuleApi")?
            .downcast::<PyType>()?;

        if inner.is_instance(module_api_cls)? {
            Ok(Self { inner })
        } else {
            Err(PyValueError::new_err(
                "Object is not a synapse.module_api.ModuleApi",
            ))
        }
    }
}

/// Convert a dict to `T` via `serde_json`, useful for implementing `parse_config`
///
/// # Errors
///
/// Returns an error if it failed to convert the dict
pub fn parse_config<'a, T: Deserialize<'a>>(config: &'a PyAny) -> PyResult<T> {
    let py = config.py();
    let config: &str = py
        .import("json")?
        .call_method1("dumps", (config,))?
        .extract()?;

    let deserializer = &mut serde_json::Deserializer::from_str(config);
    serde_path_to_error::deserialize(deserializer).map_err(|err| {
        // Figure out the path where the error happened using `serde_path_to_error`
        // XXX: This is probably good enough for now
        let path: Vec<String> = err
            .path()
            .to_string()
            .split(".")
            .map(ToOwned::to_owned)
            .collect();

        // XXX: This is ugly, but it removes the " at line X column Y" from serde_json's errors
        let mut message = err.into_inner().to_string();
        if let Some(idx) = message.rfind(" at line ") {
            message.truncate(idx);
        }

        synapse::ConfigError::new_err((message, path))
    })
}
