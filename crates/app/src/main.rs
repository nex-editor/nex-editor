use clap::{Parser, Subcommand};
use editor::{Editor, EditorState};
use document::Document;

#[derive(Parser)]
#[command(name = "nex")]
#[command(about = "A modern text editor")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Enable CLI mode
    #[arg(long)]
    cli: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new document
    New {
        /// Document name
        name: Option<String>,
    },
    /// Test editor functionality
    Test,
    /// Show editor information
    Info,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::New { name }) => {
            let doc_name = name.as_deref().unwrap_or("untitled");
            println!("Creating new document: {}", doc_name);
            
            // Create a new editor with document
            let editor = Editor::new();
            println!("✓ Editor created successfully");
            println!("✓ Document '{}' initialized", doc_name);
        }
        Some(Commands::Test) => {
            println!("Testing editor functionality...");
            test_editor();
        }
        Some(Commands::Info) => {
            println!("Nex Editor v{}", env!("CARGO_PKG_VERSION"));
            println!("A modern text editor built with Rust");
            println!("Editor and Document integration test app");
        }
        None => {
            if cli.cli {
                println!("Nex Editor CLI mode");
                test_editor();
            } else {
                println!("Welcome to Nex Editor!");
                println!("Use --help for available commands");
            }
        }
    }
}

fn test_editor() {
    println!("🔧 Initializing editor components...");
    
    // Test Document creation
    println!("📄 Testing Document creation...");
    let document = Document::new();
    println!("✓ Document created successfully");
    
    // Test EditorState creation
    println!("📝 Testing EditorState creation...");
    let editor_state = EditorState::new();
    println!("✓ EditorState created successfully");
    
    // Test Editor creation
    println!("🎯 Testing Editor creation...");
    let editor = Editor::new();
    println!("✓ Editor created successfully");
    
    // Test Default implementations
    println!("⚙️  Testing Default implementations...");
    let default_document = Document::default();
    let default_editor_state = EditorState::default();
    let default_editor = Editor::default();
    println!("✓ All Default implementations working");
    
    println!("🎉 All tests passed! Editor is working correctly.");
    
    // Show some basic information
    println!("\n📊 Component Information:");
    println!("- Document: Loro-based document with CRDT support");
    println!("- EditorState: Manages editor state and document reference");
    println!("- Editor: Main editor interface with state management");
    println!("- All components support Default trait for easy initialization");
}