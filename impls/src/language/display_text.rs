use base::prelude::{DisplayText, Language};


/// An example of how to make a display text for the labels in your app
/// so that labels are enums and translations can work
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CommonDisplayText {
    #[default]
    None,
    // --- Geometric / Dimensions ---
    Length, Width, Height, Depth, Thickness, Diameter, Radius, 
    Area, Volume, Perimeter, Circumference, Angle, Slope, 
    X, Y, Z, Global, Local, Offset,

    // --- Physical / Structural ---
    Mass, Weight, Density, Force, Moment, Torque, Pressure, Stress, 
    Strain, Stiffness, Modulus, Inertia, Temperature, Velocity, 
    Acceleration, Energy, Power, Time, Duration,

    // --- Quantities / Counting ---
    Count, Quantity, Amount, Size, Total, Sum, Average, Minimum, 
    Maximum, Range, Ratio, Percentage, Factor, Multiplier, Scale,

    // --- Identification / Metadata ---
    Name, Title, Label, Description, Comment, Tag, Category, 
    Type, Kind, Group, ID, Index, Reference, Status, Version,

    // --- UI / App Controls ---
    Settings, Preferences, Units, UnitSettings, Tools, Properties, 
    General, Advanced, View, Edit, Modify, Create, Delete, Remove, 
    Add, Update, Save, Load, Import, Export, Default, Custom,

    // --- State / Logic ---
    Enabled, Disabled, Visible, Hidden, Active, Inactive, Locked, 
    Unlocked, ReadOnly, Required, Optional, Valid, Invalid, Error, 
    Warning, Info,

    // --- Relational ---
    Parent, Child, Master, Slave, Start, End, Base, Top, Bottom, 
    Left, Right, Front, Back, Interior, Exterior,
}

impl DisplayText for CommonDisplayText {
    fn default_text(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Acceleration => "Acceleration",
            Self::Active => "Active",
            Self::Add => "Add",
            Self::Advanced => "Advanced",
            Self::Amount => "Amount",
            Self::Angle => "Angle",
            Self::Area => "Area",
            Self::Average => "Average",
            Self::Back => "Back",
            Self::Base => "Base",
            Self::Bottom => "Bottom",
            Self::Category => "Category",
            Self::Child => "Child",
            Self::Circumference => "Circumference",
            Self::Comment => "Comment",
            Self::Count => "Count",
            Self::Create => "Create",
            Self::Custom => "Custom",
            Self::Default => "Default",
            Self::Delete => "Delete",
            Self::Density => "Density",
            Self::Depth => "Depth",
            Self::Description => "Description",
            Self::Diameter => "Diameter",
            Self::Disabled => "Disabled",
            Self::Duration => "Duration",
            Self::Edit => "Edit",
            Self::Enabled => "Enabled",
            Self::End => "End",
            Self::Energy => "Energy",
            Self::Error => "Error",
            Self::Export => "Export",
            Self::Exterior => "Exterior",
            Self::Factor => "Factor",
            Self::Force => "Force",
            Self::Front => "Front",
            Self::General => "General",
            Self::Global => "Global",
            Self::Group => "Group",
            Self::Height => "Height",
            Self::Hidden => "Hidden",
            Self::ID => "ID",
            Self::Import => "Import",
            Self::Inactive => "Inactive",
            Self::Index => "Index",
            Self::Inertia => "Inertia",
            Self::Info => "Info",
            Self::Interior => "Interior",
            Self::Invalid => "Invalid",
            Self::Kind => "Kind",
            Self::Label => "Label",
            Self::Left => "Left",
            Self::Length => "Length",
            Self::Load => "Load",
            Self::Local => "Local",
            Self::Locked => "Locked",
            Self::Mass => "Mass",
            Self::Master => "Master",
            Self::Maximum => "Maximum",
            Self::Minimum => "Minimum",
            Self::Modify => "Modify",
            Self::Modulus => "Modulus",
            Self::Moment => "Moment",
            Self::Multiplier => "Multiplier",
            Self::Name => "Name",
            Self::Offset => "Offset",
            Self::Optional => "Optional",
            Self::Parent => "Parent",
            Self::Percentage => "Percentage",
            Self::Perimeter => "Perimeter",
            Self::Power => "Power",
            Self::Preferences => "Preferences",
            Self::Pressure => "Pressure",
            Self::Properties => "Properties",
            Self::Quantity => "Quantity",
            Self::Radius => "Radius",
            Self::Range => "Range",
            Self::Ratio => "Ratio",
            Self::ReadOnly => "Read Only",
            Self::Reference => "Reference",
            Self::Remove => "Remove",
            Self::Required => "Required",
            Self::Right => "Right",
            Self::Save => "Save",
            Self::Scale => "Scale",
            Self::Settings => "Settings",
            Self::Size => "Size",
            Self::Slave => "Slave",
            Self::Slope => "Slope",
            Self::Start => "Start",
            Self::Status => "Status",
            Self::Stiffness => "Stiffness",
            Self::Strain => "Strain",
            Self::Stress => "Stress",
            Self::Sum => "Sum",
            Self::Tag => "Tag",
            Self::Temperature => "Temperature",
            Self::Thickness => "Thickness",
            Self::Time => "Time",
            Self::Title => "Title",
            Self::Tools => "Tools",
            Self::Top => "Top",
            Self::Torque => "Torque",
            Self::Total => "Total",
            Self::Type => "Type",
            Self::UnitSettings => "Unit Settings",
            Self::Units => "Units",
            Self::Unlocked => "Unlocked",
            Self::Update => "Update",
            Self::Valid => "Valid",
            Self::Velocity => "Velocity",
            Self::Version => "Version",
            Self::View => "View",
            Self::Visible => "Visible",
            Self::Volume => "Volume",
            Self::Warning => "Warning",
            Self::Weight => "Weight",
            Self::Width => "Width",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            
        }
    }

    /// you do not need to translate every word and can return the 
    /// default language instead.
    fn translate<L: Language>(&self, lang: L) -> String {
        match lang.id() {
            "fr-CA" | "fr" => match self {
                Self::Width => "Largeur".to_string(),
                Self::Height => "Hauteur".to_string(),
                Self::Length => "Longueur".to_string(),
                Self::UnitSettings => "Paramètres d'unité".to_string(),
                Self::Area => "Surface".to_string(),
                Self::Volume => "Volume".to_string(),
                Self::Mass => "Masse".to_string(),
                Self::Force => "Force".to_string(),
                Self::Pressure => "Pression".to_string(),
                Self::Temperature => "Température".to_string(),
                Self::Settings => "Paramètres".to_string(),
                Self::Tools => "Outils".to_string(),
                Self::Properties => "Propriétés".to_string(),
                _ => self.default_text().to_string(),
            },
            _ => self.default_text().to_string(),
        }
    }
}

 
 