use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock, Weak},
};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;

use crate::{
    app_state::{ClusterStateRegistry, ManagerExt as _},
    cluster_discovery::ClusterDiscovery,
    frontend_commands::KubeContextSource,
    resource_rendering::{
        CrdRenderer, FallbackRenderer, ResourceColumnDefinition, ResourceRenderer,
    },
    scripting::{
        scripts_provider::{self, ScriptType, ScriptsProvider},
        types::{
            commons::ContentScript,
            resource_context_menus::ColumnTemplate,
            resource_presentations::{
                ColoredBox, ColoredBoxes, Hyperlink, RelativeTime, ResourcePresentation,
                ResourcePresentationField, Text,
            },
            ResourceRef,
        },
    },
};

struct ResourcePresentationDefinition {
    title: String,
    matcher: Option<rhai::FnPtr>,
    columns: Vec<ColumnTemplate>,
    ast: Arc<rhai::AST>,
}

pub struct ResourcePresentationFacade {
    app: tauri::AppHandle,
    scripts: RwLock<HashMap<PathBuf, ContentScript>>,
    engine: Arc<rhai::Engine>,
    registered_presentations: RwLock<Vec<ResourcePresentationDefinition>>,
}

impl ResourcePresentationFacade {
    pub fn new(app: tauri::AppHandle) -> Arc<Self> {
        Arc::new_cyclic(|weak| Self {
            app,
            engine: Self::make_engine(weak.clone()),
            scripts: RwLock::new(HashMap::new()),
            registered_presentations: RwLock::new(Vec::new()),
        })
    }

    fn make_engine(facade: Weak<Self>) -> Arc<rhai::Engine> {
        let mut engine = rhai::Engine::new();

        engine
            .build_type::<ResourceRef>()
            .build_type::<ResourcePresentation>()
            .build_type::<ColumnTemplate>()
            .build_type::<Text>()
            .build_type::<Hyperlink>()
            .build_type::<RelativeTime>()
            .build_type::<ColoredBox>()
            .build_type::<ColoredBoxes>();

        {
            engine.register_fn(
                "register_resource_presentation",
                move |ctx: rhai::NativeCallContext,
                      definition: ResourcePresentation|
                      -> Result<(), Box<rhai::EvalAltResult>> {
                    let facade = facade.upgrade().expect("facade dropped");
                    let script = ctx
                        .call_source()
                        .ok_or("only file-based scripts supported")?;

                    facade
                        .register_resource_presentation(definition, script)
                        .map_err(|e| e.to_string().into())
                },
            );
        }

        engine.set_max_expr_depths(64, 32);

        Arc::new(engine)
    }

    fn register_resource_presentation(
        &self,
        presentation: ResourcePresentation,
        script: &str,
    ) -> Result<(), ResourcePresentationError> {
        let script: PathBuf = script.into();

        let ast = self.scripts.read().unwrap();
        let ast = ast
            .get(&script)
            .unwrap()
            .ast
            .as_ref()
            .ok_or(ResourcePresentationError::PendingCompilation)?
            .as_ref()
            .map_err(|_| ResourcePresentationError::CompilationError)?;

        let mut presentations = self.registered_presentations.write().unwrap();

        presentations.push(ResourcePresentationDefinition {
            title: presentation.title,
            matcher: presentation.matcher,
            columns: presentation.columns,
            ast: Arc::clone(ast),
        });

        Ok(())
    }

    /// Returns the names of all available renderers for the given GVK
    pub async fn get_renderers(
        &self,
        context_source: &KubeContextSource,
        gvk: &GroupVersionKind,
    ) -> Result<Vec<String>, ResourcePresentationError> {
        let clusters = self.app.state::<ClusterStateRegistry>();
        let discovery = clusters.discovery_for(context_source).unwrap();

        let registered_presentations = self.registered_presentations.read().unwrap();

        let crds: Vec<GroupVersionKind> = match &*discovery {
            ClusterDiscovery::Inflight(_) => {
                return Ok(vec![]);
            }
            ClusterDiscovery::Completed(resources) => resources.crds.keys().cloned().collect(),
        };

        let renderers = registered_presentations
            .iter()
            .filter(|&presentation| {
                presentation
                    .matcher
                    .as_ref()
                    .map(|matcher| {
                        let gvk = gvk.clone();
                        matcher.call::<bool>(
                            &self.engine,
                            &Arc::clone(&presentation.ast),
                            (gvk.group, gvk.version, gvk.kind),
                        )
                    })
                    .unwrap_or(Ok(true))
                    .map_err(ResourcePresentationError::Matcher)
                    .expect("handle me")
            })
            .map(|presentation| presentation.title.clone())
            .chain({
                crds.contains(gvk)
                    .then(|| "Custom resource default".to_owned())
                    .into_iter()
            })
            .chain(std::iter::once("Simple list".to_owned()))
            .collect();

        Ok(renderers)
    }

    pub async fn get_renderer(
        &self,
        _gvk: &GroupVersionKind,
        presentation: &str,
    ) -> Box<dyn ResourceRenderer> {
        let generic_renderer = FallbackRenderer {};
        let crd_renderer = CrdRenderer {};

        if presentation == generic_renderer.display_name() {
            return Box::new(generic_renderer) as Box<dyn ResourceRenderer>;
        } else if presentation == crd_renderer.display_name() {
            return Box::new(crd_renderer) as Box<dyn ResourceRenderer>;
        }

        let registered_presentations = self.registered_presentations.read().unwrap();

        let presentation = registered_presentations
            .iter()
            .find(|p| p.title == presentation);

        if presentation.is_none() {
            return Box::new(generic_renderer) as Box<dyn ResourceRenderer>;
        }

        let presentation = presentation.unwrap();

        Box::new(ScriptedRenderer {
            title: presentation.title.clone(),
            templates: presentation.columns.clone(),
            engine: Arc::clone(&self.engine),
            ast: Arc::clone(&presentation.ast),
        }) as Box<dyn ResourceRenderer>
    }

    pub fn evaluate(
        &self,
        scripts_provider: &ScriptsProvider,
    ) -> Result<(), ResourcePresentationError> {
        let menu_scripts = scripts_provider
            .get_scripts_for_type(ScriptType::Presentation)
            .unwrap();

        for entrypoint in &menu_scripts {
            tracing::info!("Evaluating {}", entrypoint.to_string_lossy());

            let ast_arc = {
                let mut scripts = self.scripts.write().unwrap();

                let script = scripts
                    .entry(entrypoint.to_owned())
                    .or_insert(ContentScript::new());

                let ast_result = script.ast.get_or_insert_with(|| {
                    self.engine.compile_file(entrypoint.clone()).map(Arc::new)
                });

                ast_result
                    .as_ref()
                    .map_err(|_| ResourcePresentationError::CompilationError)?
                    .clone()
            };

            self.engine.eval_ast::<()>(&ast_arc)?;
        }

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ResourcePresentationError {
    #[error("Error evaluating script: {0}")]
    EvaluationResult(#[from] Box<rhai::EvalAltResult>),

    #[error("The script has not yet been compiled")]
    PendingCompilation,

    #[error("The script has a compilation error")]
    CompilationError,

    #[error(transparent)]
    ScriptDirectoryResolution(#[from] scripts_provider::Error),

    #[error("Call to matcher failed: {0}")]
    Matcher(Box<rhai::EvalAltResult>),
}

struct ScriptedRenderer {
    title: String,
    templates: Vec<ColumnTemplate>,
    engine: Arc<rhai::Engine>,
    ast: Arc<rhai::AST>,
}

impl ResourceRenderer for ScriptedRenderer {
    fn display_name(&self) -> &str {
        &self.title
    }

    fn column_definitions(
        &self,
        _gvk: &GroupVersionKind,
        _crd: Option<&k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition>,
    ) -> Result<
        Vec<crate::resource_rendering::ResourceColumnDefinition>,
        crate::frontend_types::BackendError,
    > {
        Ok(self
            .templates
            .iter()
            .map(|t| ResourceColumnDefinition {
                title: t.title.clone(),
                filterable: true,
            })
            .collect())
    }

    fn render(
        &self,
        _gvk: &GroupVersionKind,
        _crd: Option<&CustomResourceDefinition>,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<ResourcePresentationField, String>>, crate::frontend_types::BackendError>
    {
        Ok(self
            .templates
            .iter()
            .map(|t| {
                let obj = rhai::serde::to_dynamic(obj)
                    .expect("failed to convert Kubernetes resource to dynamic");

                t.render
                    .call::<rhai::Dynamic>(&self.engine, &self.ast, (obj,))
                    .map_err(|e| e.to_string())
                    .map(|value| {
                        if value.is::<Text>() {
                            return ResourcePresentationField::Text(value.cast::<Text>());
                        }

                        if value.is::<Hyperlink>() {
                            return ResourcePresentationField::Hyperlink(value.cast::<Hyperlink>());
                        }

                        if value.is::<RelativeTime>() {
                            return ResourcePresentationField::RelativeTime(
                                value.cast::<RelativeTime>(),
                            );
                        }

                        if value.is::<ColoredBox>() {
                            return ResourcePresentationField::ColoredBox(
                                value.cast::<ColoredBox>(),
                            );
                        }

                        if value.is::<ColoredBoxes>() {
                            return ResourcePresentationField::ColoredBoxes(
                                value.cast::<ColoredBoxes>(),
                            );
                        }

                        ResourcePresentationField::Text(Text {
                            content: value.to_string(),
                            properties: None,
                        })
                    })
            })
            .collect())
    }
}
