# Quality Control Room

A comprehensive suite of quality control tools for industrial production and inspection processes.

🌐 **Live Demo**: [https://quality-control.io](https://quality-control.io)

![Quality Control Room Interface](doc/images/quality-screen.png)
*Example analysis results in test mode showing the hybrid ML + statistical approach in action*

## Overview

Quality Control Room provides a hybrid approach to quality control in manufacturing and production environments, combining classical statistical methods with modern machine learning techniques. This toolset is designed to help industrial engineers, quality assurance professionals, and production managers maintain and improve product quality through data-driven insights.

## Key Features

### Classical Statistical Methods
- Traditional statistical process control (SPC) metrics
- Control charts and process capability analysis
- Standard quality control calculations and measurements

### Machine Learning Enhanced Approach
- **Hybrid ML + Statistical Method**: An innovative approach that combines machine learning with statistical analysis
- **No Normality Assumption Required**: Unlike traditional methods, this approach doesn't require data to follow a normal distribution
- **Small Sample Optimization**: Works effectively with limited sample sizes, addressing a common limitation of classical statistical approaches
- **Improved Accuracy**: Better or equal performance than traditional methods, especially for small sampling scenarios

## Advantages

✅ **Works with small sample sizes** - Traditional methods often require large datasets  
✅ **No normality hypothesis required** - More flexible than classical statistical approaches  
✅ **Hybrid methodology** - Combines the reliability of statistical methods with the power of machine learning  
✅ **Industrial-grade** - Designed for real-world industrial production environments  

## Project Structure

```
├── engine/          # Core Rust-based processing engine
├── doc/             # Detailed documentation (under construction!)
├── ui/              # Vue.js web interface
├── data/            # Sample data and configuration files
├── lib/             # Compiled libraries (XGBoost wrapper)
├── xgbwrapper/      # C wrapper for XGBoost integration
├── vscode_ext/      # Simple VS Code extension to run tasks
└── systemd/         # System service configuration
```

## Technology Stack

*Yes, there's life beyond Python!* 😄

- **Backend Engine**: Rust (high-performance computing)
- **Machine Learning**: XGBoost with custom C wrapper
- **Frontend**: Vue.js with Tailwind CSS
- **Deployment**: Systemd service integration

## Current Status

🚧 **Under Active Development** 🚧

This project is currently under construction and is being developed incrementally during my spare time. Many features are planned but not yet implemented. The core hybrid ML + statistical methodology is functional, but the full feature set is still being built out.

### What's Working
- Core ML + statistical analysis engine
- Basic web interface for continuous tools (Beta distribution approximation)
- XGBoost integration for distribution parameter predictions
- Sample data processing capabilities (most useful statistical metrics)

### Planned Features
- More tools for normal approximation and discrete parameters
- Extended statistical method library
- Advanced visualization tools
- Industrial IoT integration
- Comprehensive reporting system
- LLM agent "Virtual Engineer" to help with results interpretation and report generation
- Multi-language support

## Documentation

Detailed documentation is currently being prepared and will be available soon. It will include:

- Complete methodology explanation
- API reference
- Usage examples and tutorials
- Comparison studies with traditional methods
- Something else

## Getting Started

*Installation and setup instructions will be provided once the core features are stabilized.*

## Contributing

This is a personal project developed in my spare time. While I'm not actively seeking contributions at this early stage, feedback and suggestions are always welcome through issues or contacts!

## License

See [LICENSE](LICENSE) file for details.

## Contact

For questions, suggestions, or industrial collaboration opportunities, please contact me:

- **Email**: piyadov@alumni.usp.br
- **LinkedIn**: [Vasilii Piiadov](https://www.linkedin.com/in/vasilii-piiadov/)

---

*Quality Control Room - Bridging traditional quality control with modern machine learning for better industrial production control.*
