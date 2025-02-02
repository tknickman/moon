use crate::language_platform::PlatformType;
use crate::project::{PartialTaskOptionsConfig, TaskOptionsConfig};
use crate::validate::validate_portable_paths;
use moon_common::cacheable;
use moon_target::{Target, TargetScope};
use rustc_hash::FxHashMap;
use schematic::{
    derive_enum, Config, ConfigEnum, ConfigError, ConfigLoader, Format, Segment, ValidateError,
};

fn validate_command<D, C>(cmd: &TaskCommandArgs, _task: &D, _ctx: &C) -> Result<(), ValidateError> {
    // Only fail for empty strings and not `None`
    let empty = match cmd {
        TaskCommandArgs::None => false,
        TaskCommandArgs::String(cmd_string) => {
            let mut parts = cmd_string.split(' ');

            if let Some(part) = parts.next() {
                part.is_empty()
            } else {
                true
            }
        }
        TaskCommandArgs::Sequence(cmd_args) => cmd_args.is_empty() || cmd_args[0].is_empty(),
    };

    if empty {
        return Err(ValidateError::new(
            "a command is required; use \"noop\" otherwise",
        ));
    }

    Ok(())
}

pub fn validate_deps<D, C>(deps: &[Target], _data: &D, _ctx: &C) -> Result<(), ValidateError> {
    for (i, dep) in deps.iter().enumerate() {
        if matches!(dep.scope, TargetScope::All | TargetScope::Tag(_)) {
            return Err(ValidateError::with_segment(
                "target scope not supported as a task dependency",
                Segment::Index(i),
            ));
        }
    }

    Ok(())
}

derive_enum!(
    #[derive(ConfigEnum, Copy, Default)]
    pub enum TaskType {
        Build,
        Run,
        #[default]
        Test,
    }
);

derive_enum!(
    #[derive(Default)]
    #[serde(untagged, expecting = "expected a string or a sequence of strings")]
    pub enum TaskCommandArgs {
        #[default]
        None,
        String(String),
        Sequence(Vec<String>),
    }
);

cacheable!(
    #[derive(Clone, Config, Debug, Eq, PartialEq)]
    pub struct TaskConfig {
        #[setting(validate = validate_command)]
        pub command: TaskCommandArgs,

        pub args: TaskCommandArgs,

        #[setting(validate = validate_deps)]
        pub deps: Vec<Target>,

        pub env: FxHashMap<String, String>,

        // TODO
        #[setting(skip)]
        pub global_inputs: Vec<String>,

        // None = All inputs (**/*)
        // [] = No inputs
        // [...] = Specific inputs
        #[setting(validate = validate_portable_paths)]
        pub inputs: Option<Vec<String>>,

        pub local: bool,

        #[setting(validate = validate_portable_paths)]
        pub outputs: Option<Vec<String>>,

        #[setting(nested)]
        pub options: TaskOptionsConfig,

        pub platform: PlatformType,

        #[serde(rename = "type")]
        pub type_of: Option<TaskType>,
    }
);

impl TaskConfig {
    pub fn parse<T: AsRef<str>>(code: T) -> Result<TaskConfig, ConfigError> {
        let result = ConfigLoader::<TaskConfig>::new()
            .code(code.as_ref(), Format::Yaml)?
            .load()?;

        Ok(result.config)
    }
}
