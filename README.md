# typst-web

## Description
typst-web is a web server that takes a Typst template and renders it. It allows users to upload Typst templates and receive rendered documents in return.

## Installation
To install typst-web, follow these steps:

1. Clone the repository:
    ```bash
    git clone https://github.com/simoneromano96/typst-web.git
    ```
2. Navigate to the project directory:
    ```bash
    cd typst-web
    ```
3. Build the project using Cargo:
    ```bash
    cargo build --release
    ```

## Usage
To start the server, run:
```bash
./target/release/typst-web
```
The server will start on `http://localhost:3030`. You can upload your Typst templates via the web interface and receive the rendered documents.

## Docker
You can also run the server using Docker:

1. Build the Docker image:
    ```bash
    docker build -t typst-web .
    ```
2. Run the Docker container:
    ```bash
    docker run -p 3030:3030 typst-web
    ```

## Contributing
Contributions are welcome! Please follow these steps to contribute:

1. Fork the repository.
2. Create a new branch:
    ```bash
    git checkout -b feature-branch
    ```
3. Make your changes and commit them:
    ```bash
    git commit -m "Description of changes"
    ```
4. Push to the branch:
    ```bash
    git push origin feature-branch
    ```
5. Create a pull request.

## License
This project is licensed under either of the following licenses, at your option:
- MIT License
- Apache License, Version 2.0

## Roadmap
Here are some possible future developments for typst-web:

- **Authentication and Authorization**: Implement user authentication and authorization to secure the API endpoints.
- **Enhanced Error Handling**: Improve error handling to provide more detailed and user-friendly error messages.
- **Performance Optimization**: Optimize the rendering process to handle larger templates and more concurrent requests.
- **Web Interface**: Add a web interface for a better user experience, including template previews and editing capabilities.
