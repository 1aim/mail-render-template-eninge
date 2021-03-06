use tera_crate::{Tera, TesterFn, FilterFn, GlobalFn};
use serde::Serialize;

use ::traits::{RenderEngine, RenderEngineBase, AdditionalCIds};
use ::spec::{TemplateSpec, SubTemplateSpec, TemplateSource};

use self::error::TeraError;

pub mod error;

pub struct TeraRenderEngine {
    tera: Tera
}

impl TeraRenderEngine {

    /// create a new TeraRenderEngine given a base_templates_dir
    ///
    /// The `base_templates_glob` contains a number of tera templates which can be used to
    /// inherit (or include) from e.g. a `base_mail.html` which is then used in all
    /// `mail.html` templates through `{% extends "base_mail.html" %}`.
    ///
    /// The `base_templates_glob` _is separate from template dirs used by
    /// the `RenderTemplateEngine`_. It contains only tera templates to be reused at
    /// other places.
    ///
    pub fn new(base_templats_glob: &str) -> Result<Self, TeraError> {
        let tera = Tera::new(base_templats_glob)?;

        Ok(TeraRenderEngine { tera })
    }

    /// expose `Tera::register_filter`
    pub fn register_filter(&mut self, name: &str, filter: FilterFn) {
        self.tera.register_filter(name, filter);
    }

    /// exposes `Tera::register_tester`
    pub fn register_tester(&mut self, name: &str, tester: TesterFn) {
        self.tera.register_tester(name, tester);
    }

    /// exposes `Tera::register_global_function`
    pub fn register_global_function(&mut self, name: &str, function: GlobalFn) {
        self.tera.register_global_function(name, function)
    }

    /// exposes `Tera::autoescape_on`
    pub fn set_autoescape_file_suffixes(&mut self, suffixes: Vec<&'static str>) {
        self.tera.autoescape_on(suffixes)
    }

}

impl RenderEngineBase for TeraRenderEngine {
    // nothing gurantees that the templates use \r\n, so by default fix newlines
    // but it can be disabled
    const PRODUCES_VALID_NEWLINES: bool = false;

    type RenderError = TeraError;
    type LoadingError = TeraError;

    fn load_templates(&mut self, spec: &TemplateSpec) -> Result<(), Self::LoadingError> {
        implement_load_helper! {
            input::<Tera>(spec, &mut self.tera);
            error(TeraError);
            collision_error_fn(|id| { TeraError::TemplateIdCollision { id } });
            has_template_fn(|tera, id| { tera.templates.contains_key(id) });
            remove_fn(|tera, id| { tera.templates.remove(*id) });
            add_file_fn(|tera, path| { Ok(tera.add_template_file(path, None)?) });
            add_content_fn(|tera, id, content| { Ok(tera.add_raw_template(id, content)?) });
        }
    }


    /// This can be used to reload a templates.
    fn unload_templates(&mut self, spec: &TemplateSpec) {
        for sub_spec in spec.sub_specs() {
            let id = sub_spec.source().id();
            self.tera.templates.remove(id);
        }
    }


    fn unknown_template_id_error(id: &str) -> Self::RenderError {
        TeraError::UnknowTemplateId { id: id.to_owned() }
    }
}


#[derive(Serialize)]
struct DataWrapper<'a,D: Serialize + 'a> {
    data: &'a D,
    cids: AdditionalCIds<'a>
}

impl<D> RenderEngine<D> for TeraRenderEngine
    where D: Serialize
{
    fn render(
        &self,
        spec: &SubTemplateSpec,
        data: &D,
        cids: AdditionalCIds
    ) -> Result<String, Self::RenderError> {
        let data = &DataWrapper { data, cids };
        let id = spec.source().id();
        Ok(self.tera.render(id, data)?)
    }
}

