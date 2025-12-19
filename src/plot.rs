use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::budget::LinkBudget;

pub fn generate_html_summary(
    budget: &LinkBudget,
    output_path_str: &str,
) -> Result<(), std::io::Error> {
    let path = Path::new(output_path_str);
    let mut file = File::create(path)?;

    let svg = generate_svg(budget);

    writeln!(
        file,
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Link Budget - {}</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            margin: 2rem;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            text-align: center;
            margin-bottom: 2rem;
        }}
        .diagram {{
            width: 100%;
            overflow-x: auto;
            display: flex;
            justify-content: center;
        }}
        svg {{
            max-width: 100%;
            height: auto;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Link Budget: {}</h1>
        <div class="diagram">
            {}
        </div>
    </div>
</body>
</html>"##,
        budget.name, budget.name, svg
    )?;

    Ok(())
}

fn generate_svg(budget: &LinkBudget) -> String {
    let width = 800;
    let height = 400;

    // Positions
    let tx_x = 100;
    let rx_x = 600;
    let component_y = 150;
    let component_width = 120;
    let component_height = 80;

    // Calculations for display
    let path_loss = budget.path_loss();
    let rx_power = budget.pin_at_receiver();
    let snr = budget.snr();
    let margin = budget.frequency_dependent_loss.unwrap_or(0.0);

    let mut svg = String::new();

    // Header
    svg.push_str(&format!(
        r##"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"##,
        width, height, width, height
    ));

    // Definitions for markers
    svg.push_str(r##"
    <defs>
        <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
            <path d="M0,0 L0,6 L9,3 z" fill="#666" />
        </marker>
    </defs>
    "##);

    // Connecting Line
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#666" stroke-width="2" marker-end="url(#arrow)" />"##,
        tx_x + component_width,
        component_y + component_height / 2,
        rx_x,
        component_y + component_height / 2
    ));

    // Transmitter Box
    svg.push_str(&format!(
        r##"
        <g transform="translate({}, {})">
            <rect width="{}" height="{}" rx="5" ry="5" fill="#e3f2fd" stroke="#2196f3" stroke-width="2"/>
            <text x="{}" y="25" text-anchor="middle" font-weight="bold" fill="#1565c0">Transmitter</text>
            <text x="10" y="45" font-size="12" fill="#333">Power: {:.1} dBm</text>
            <text x="10" y="60" font-size="12" fill="#333">Gain: {:.1} dB</text>
        </g>
        "##,
        tx_x, component_y,
        component_width, component_height,
        component_width / 2,
        budget.transmitter.output_power,
        budget.transmitter.gain
    ));

    // Receiver Box
    svg.push_str(&format!(
        r##"
        <g transform="translate({}, {})">
            <rect width="{}" height="{}" rx="5" ry="5" fill="#e8f5e9" stroke="#4caf50" stroke-width="2"/>
            <text x="{}" y="25" text-anchor="middle" font-weight="bold" fill="#2e7d32">Receiver</text>
            <text x="10" y="45" font-size="12" fill="#333">Gain: {:.1} dB</text>
            <text x="10" y="60" font-size="12" fill="#333">NF: {:.1} dB</text>
        </g>
        "##,
        rx_x, component_y,
        component_width, component_height,
        component_width / 2,
        budget.receiver.gain,
        budget.receiver.noise_figure
    ));

    // Path Loss Label
    svg.push_str(&format!(
        r##"
        <g transform="translate({}, {})">
            <text x="0" y="0" text-anchor="middle" font-size="12" fill="#666">Path Loss</text>
            <text x="0" y="15" text-anchor="middle" font-weight="bold" fill="#d32f2f">{:.1} dB</text>
            <text x="0" y="30" text-anchor="middle" font-size="10" fill="#666">Margin: {:.1} dB</text>
        </g>
        "##,
        (tx_x + rx_x + component_width) / 2,
        component_y + component_height / 2 - 20,
        path_loss,
        margin
    ));

    // Result Stats
    svg.push_str(&format!(
        r##"
        <g transform="translate({}, {})">
            <text x="0" y="0" text-anchor="middle" font-weight="bold" fill="#333">Results</text>
            <text x="0" y="20" text-anchor="middle" font-size="14" fill="#333">Rx Power: {:.1} dBm</text>
            <text x="0" y="40" text-anchor="middle" font-size="14" fill="#333">SNR: {:.1} dB</text>
        </g>
        "##,
        width / 2,
        height - 50,
        rx_power,
        snr
    ));

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::LinkBudget;
    use crate::path_loss::PathLoss;
    use crate::receiver::Receiver;
    use crate::transmitter::Transmitter;

    #[test]
    fn test_generate_html() {
        let budget = LinkBudget {
            name: "Test Link",
            bandwidth: 10e6,
            transmitter: Transmitter {
                output_power: 40.0,
                gain: 10.0,
                bandwidth: 10e6,
            },
            receiver: Receiver {
                gain: 20.0,
                temperature: 290.0,
                noise_figure: 5.0,
                bandwidth: 10e6,
            },
            path_loss: PathLoss {
                frequency: 2.4e9,
                distance: 1000.0,
            },
            frequency_dependent_loss: Some(3.0),
        };

        let output_path = "target/test_link_budget.html";
        let html_result = generate_html_summary(&budget, output_path);
        assert!(html_result.is_ok());

        let content = std::fs::read_to_string(output_path).unwrap();
        assert!(content.contains("Test Link"));
        assert!(content.contains("<svg"));
        assert!(content.contains("Transmitter"));
        assert!(content.contains("Receiver"));
    }
}
