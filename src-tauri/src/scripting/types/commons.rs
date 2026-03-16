use std::sync::Arc;

use rhai::{EvalAltResult, FnPtr, AST};

pub struct ContentScript {
    /// If Some, holds the compilation result of the script
    pub ast: Option<Result<Arc<AST>, Box<EvalAltResult>>>,
}

impl ContentScript {
    pub fn new() -> Self {
        Self { ast: None }
    }
}

#[derive(Debug, Clone)]
pub struct FnPtrWithAst {
    pub fnptr: FnPtr,
    pub ast: Arc<AST>,
}

impl FnPtrWithAst {
    pub fn new(fnptr: FnPtr, ast: Arc<AST>) -> Self {
        Self { fnptr, ast }
    }
}

#[derive(Clone, Debug)]
pub struct CallbackContext {
    pub app_handle: tauri::AppHandle,
    pub frontend_tab: String,
}

impl CallbackContext {
    pub fn new(app_handle: tauri::AppHandle, frontend_tab: String) -> Self {
        Self {
            app_handle,
            frontend_tab,
        }
    }
}
