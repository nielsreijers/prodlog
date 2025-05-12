pub const OUTPUT_CSS: &str = r#"
    <style>
        :root {
            --proton-blue: #6D4AFF;
            --proton-blue-hover: rgb(206, 198, 236);
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
        .command-output { 
            white-space: pre-wrap;
            margin: 0;
            padding: 1.5rem;
            background-color: var(--proton-background);
            border: 1px solid var(--proton-border);
            border-radius: 12px;
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
            font-size: 0.875rem;
            line-height: 1.5;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }
        .match-highlight { 
            background-color: rgba(109, 74, 255, 0.1);
            color: var(--proton-blue);
            padding: 0.125rem 0.25rem;
            border-radius: 4px;
        }
        .diff-del {
            background: #ffebee;
            color: #b71c1c;
            padding: 0.125rem 0;
        }
        .diff-ins {
            background: #e8f5e9;
            color: #1b5e20;
            padding: 0.125rem 0;
        }
        .diff-del span, .diff-ins span {
            font-weight: bold;
            margin-right: 0.5em;
            color: inherit;
        }
        pre {
            margin: 0;
            font-family: inherit;
            font-size: inherit;
            line-height: inherit;
        }
        form {
            background-color: var(--proton-background);
            padding: 1.5rem;
            border-radius: 12px;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            margin-top: 1.5rem;
        }
        textarea {
            width: 100%;
            padding: 0.75rem 1rem;
            border: 1px solid var(--proton-border);
            border-radius: 8px;
            font-size: 0.875rem;
            color: var(--proton-text);
            background-color: var(--proton-background);
            transition: all 0.2s ease;
            font-family: inherit;
            resize: vertical;
        }
        textarea:focus {
            outline: none;
            border-color: var(--proton-blue);
            box-shadow: 0 0 0 2px rgba(109, 74, 255, 0.1);
        }
        .button-group {
            display: flex;
            gap: 1rem;
            margin-top: 1rem;
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
        .button {
            padding: 0.75rem 1.5rem;
            background-color: transparent;
            color: var(--proton-text);
            border: 1px solid var(--proton-border);
            border-radius: 8px;
            text-decoration: none;
            font-weight: 500;
            transition: all 0.2s ease;
            display: inline-block;
        }
        .button:hover {
            background-color: var(--proton-hover);
        }
        .switch-container {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            margin: 1rem 0;
        }
        .switch {
            position: relative;
            display: inline-block;
            width: 40px;
            height: 20px;
        }
        .switch input {
            opacity: 0;
            width: 0;
            height: 0;
        }
        .slider {
            position: absolute;
            cursor: pointer;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: var(--proton-border);
            transition: .4s;
            border-radius: 20px;
        }
        .slider:before {
            position: absolute;
            content: "";
            height: 16px;
            width: 16px;
            left: 2px;
            bottom: 2px;
            background-color: white;
            transition: .4s;
            border-radius: 50%;
        }
        input:checked + .slider {
            background-color: var(--proton-blue);
        }
        input:checked + .slider:before {
            transform: translateX(20px);
        }
        .switch-label {
            font-size: 0.875rem;
            color: var(--proton-text-secondary);
        }
        .header-info {
            background-color: var(--proton-background);
            padding: 1.5rem;
            border-radius: 12px;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            margin-bottom: 1.5rem;
        }
        .info-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1rem;
            margin-bottom: 1rem;
        }
        .info-item {
            display: flex;
            gap: 0.5rem;
            align-items: baseline;
        }
        .info-label {
            color: var(--proton-text-secondary);
            font-size: 0.875rem;
            min-width: 80px;
        }
        .info-value {
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
            font-size: 0.875rem;
            word-break: break-all;
        }
        .message {
            margin-top: 1rem;
            padding: 1rem;
            background-color: var(--proton-hover);
            border-radius: 8px;
            font-style: italic;
            color: var(--proton-text-secondary);
        }
        .content-box {
            background-color: var(--proton-background);
            border-radius: 12px;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            overflow: hidden;
        }
        h2 {
            margin: 1rem 0;
            font-size: 1.25rem;
            color: var(--proton-text);
        }
        .form-group {
            margin-bottom: 1rem;
        }
        .form-group label {
            display: block;
            margin-bottom: 0.5rem;
            color: var(--proton-text-secondary);
            font-size: 0.875rem;
        }
    </style>
"#;

pub const MAIN_CSS: &str = r#"
    <style>
        :root {
            --proton-blue: #6D4AFF;
            --proton-blue-hover:rgb(206, 198, 236);
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
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        tbody {
            border-bottom: 1px solid var(--proton-border);
        }
        tbody:hover {
            background-color: var(--proton-blue-hover);
        }
        .message-row .message {
            font-size: 0.875rem;
            font-style: italic;
            color: var(--proton-text-secondary);
            padding-top: 0;
            padding-bottom: 1rem;
        }
        .message-row td {
            border-top: none;
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
        tr.noop-row {
            background-color: #aaaaaa !important;
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
        .switch-container {
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }
        .switch {
            position: relative;
            display: inline-block;
            width: 40px;
            height: 20px;
        }
        .switch input {
            opacity: 0;
            width: 0;
            height: 0;
        }
        .slider {
            position: absolute;
            cursor: pointer;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: var(--proton-border);
            transition: .4s;
            border-radius: 20px;
        }
        .slider:before {
            position: absolute;
            content: "";
            height: 16px;
            width: 16px;
            left: 2px;
            bottom: 2px;
            background-color: white;
            transition: .4s;
            border-radius: 50%;
        }
        input:checked + .slider {
            background-color: var(--proton-blue);
        }
        input:checked + .slider:before {
            transform: translateX(20px);
        }
        .switch-label {
            font-size: 0.875rem;
            color: var(--proton-text-secondary);
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

pub const EDIT_ICON_SVG: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
                    <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
                </svg>
"#;
