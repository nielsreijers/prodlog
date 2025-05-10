pub const OUTPUT_CSS: &str = r#"
    <style>
        :root {
            --proton-blue: #6D4AFF;
            --proton-background: #1C1B1F;
            --proton-text: #FFFFFF;
            --proton-text-secondary: #A0A0A0;
            --proton-border: #2D2D2D;
        }
        body { 
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
            margin: 0;
            padding: 0;
            background-color: var(--proton-background);
            color: var(--proton-text);
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }
        .command-output { 
            white-space: pre-wrap;
            margin: 0;
            padding: 1.5rem;
            background-color: rgba(255, 255, 255, 0.05);
            border-radius: 12px;
            font-size: 0.875rem;
            line-height: 1.5;
        }
        .back-link { 
            margin-bottom: 1.5rem; 
        }
        .back-link a {
            color: var(--proton-text-secondary);
            text-decoration: none;
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            font-size: 0.875rem;
            transition: color 0.2s ease;
        }
        .back-link a:hover {
            color: var(--proton-text);
        }
        .match-highlight { 
            background-color: #ffeb3b;
            color: #222;
            padding: 2px 4px;
            border-radius: 4px;
            font-weight: bold;
            box-shadow: 0 0 0 2px #fff59d;
        }
        .diff-del {
            background: #ffebee;
            color: #b71c1c;
        }
        .diff-ins {
            background: #e8f5e9;
            color: #1b5e20;
        }
        .diff-del span, .diff-ins span {
            font-weight: bold;
            margin-right: 0.5em;
        }
    </style>
"#;

pub const MAIN_CSS: &str = r#"
    <style>
        :root {
            --proton-blue: #6D4AFF;
            --proton-blue-hover: #7B5AFF;
            --proton-background: #FFFFFF;
            --proton-text: #1C1B1F;
            --proton-text-secondary: #4E4B66;
            --proton-border: #E5E7EB;
            --proton-hover: #F5F5F5;
        }
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            margin: 0;
            padding: 0;
            background-color: var(--proton-background);
            color: var(--proton-text);
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
            transition: max-width 0.3s ease;
        }
        .container.full-width {
            max-width: none;
            padding: 2rem;
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 2rem;
        }
        .view-toggle {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.75rem 1.5rem;
            background-color: var(--proton-blue);
            color: white;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-weight: 500;
            transition: all 0.2s ease;
        }
        .view-toggle:hover {
            background-color: var(--proton-blue-hover);
        }
        .view-toggle svg {
            width: 16px;
            height: 16px;
            stroke: currentColor;
        }
        h1 {
            color: var(--proton-text);
            font-size: 2rem;
            margin-bottom: 2rem;
            font-weight: 600;
        }
        .filters {
            background-color: var(--proton-background);
            padding: 1.5rem;
            border-radius: 12px;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            margin-bottom: 2rem;
        }
        .filters form {
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
            align-items: center;
        }
        input, select {
            padding: 0.75rem 1rem;
            border: 1px solid var(--proton-border);
            border-radius: 8px;
            font-size: 0.875rem;
            color: var(--proton-text);
            background-color: var(--proton-background);
            transition: all 0.2s ease;
        }
        input:focus, select:focus {
            outline: none;
            border-color: var(--proton-blue);
            box-shadow: 0 0 0 2px rgba(109, 74, 255, 0.1);
        }
        button {
            padding: 0.75rem 1.5rem;
            background-color: var(--proton-blue);
            color: white;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-weight: 500;
            transition: all 0.2s ease;
        }
        button:hover {
            background-color: var(--proton-blue-hover);
        }
        button[type="button"] {
            background-color: transparent;
            color: var(--proton-text);
            border: 1px solid var(--proton-border);
        }
        button[type="button"]:hover {
            background-color: var(--proton-hover);
        }
        table {
            width: 100%;
            border-collapse: separate;
            border-spacing: 0;
            margin-top: 1rem;
            background-color: var(--proton-background);
            border-radius: 12px;
            overflow: hidden;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            table-layout: fixed;
        }
        th, td {
            padding: 1rem;
            text-align: left;
            border-bottom: 1px solid var(--proton-border);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        th {
            background-color: var(--proton-hover);
            font-weight: 600;
            color: var(--proton-text-secondary);
        }
        tr:hover {
            background-color: var(--proton-hover);
        }
        a {
            color: var(--proton-blue);
            text-decoration: none;
            font-weight: 500;
        }
        a:hover {
            text-decoration: underline;
        }
        .output-preview {
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
            margin-top: 0.5rem;
            padding: 0.75rem;
            background-color: var(--proton-hover);
            border: 1px solid var(--proton-border);
            border-radius: 8px;
            white-space: pre-wrap;
            max-height: 100px;
            overflow-y: auto;
            font-size: 0.875rem;
        }
        .match-highlight {
            background-color: rgba(109, 74, 255, 0.1);
            color: var(--proton-blue);
            padding: 0.125rem 0.25rem;
            border-radius: 4px;
        }
        tr.error-row {
            background-color: #ffeaea !important;
        }
        .copy-button {
            background: none;
            border: none;
            color: var(--proton-text-secondary);
            cursor: pointer;
            padding: 0.25rem;
            margin-left: 0.5rem;
            border-radius: 4px;
            transition: all 0.2s ease;
        }
        .copy-button:hover {
            background-color: var(--proton-hover);
            color: var(--proton-text);
        }
        .copy-button svg {
            width: 16px;
            height: 16px;
        }
        .copy-button.copied {
            color: var(--proton-blue);
        }
    </style>
    "#;

pub const CAPTURE_TYPE_RUN_SVG: &str = r#"<svg fill="none" stroke="currentColor" stroke-width="1" width="24" height="24">
                    <path d="M4 6l4 4-4 4" stroke-linecap="round"/>
                    <path d="M12 14h6" stroke-linecap="round"/>
                </svg>"#;

pub const CAPTURE_TYPE_EDIT_SVG: &str = r#"<svg fill="none" stroke="currentColor" stroke-width="1" width="16" height="16">
                    <path d="M8 1h4a2 2 0 012 2v10a2 2 0 01-2 2H4a2 2 0 01-2-2V3a2 2 0 012-2z"/>
                    <path d="M4 4h7 M4 7h7 M4 10h5"/>
                </svg>"#;

pub const COPY_ICON_SVG: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M8 4v12a2 2 0 002 2h8a2 2 0 002-2V7.242a2 2 0 00-.602-1.43L16.083 2.57A2 2 0 0014.685 2H10a2 2 0 00-2 2z"/>
                    <path d="M16 18v2a2 2 0 01-2 2H6a2 2 0 01-2-2V9a2 2 0 012-2h2"/>
                </svg>
"#;                
