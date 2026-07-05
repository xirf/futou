# Nginx

Nginx from [nginx.org](https://nginx.org/en/download.html).

## Versions

| Version | Source |
|---------|--------|
| 1.27.4 | Windows |
| 1.26.3 | Windows |

## Starting

```bash
futou start nginx 1.27.4
```

When starting from the GUI, a document root prompt appears. The daemon generates `nginx.conf` and launches Nginx:

```nginx
server {
    listen 80;
    server_name localhost;
    root "D:/projects/myapp";
    index index.html;

    location / {
        try_files $uri $uri/ =404;
    }
}
```

MIME types are included inline — no external `mime.types` dependency.

The generated config is saved to `%APPDATA%\.futou\runtimes\nginx\<version>\data\conf\nginx.conf`. **It is only generated on first start** — subsequent starts preserve your edits.

## Configuration

The GUI Config button opens the generated `nginx.conf`. Common customizations:

```nginx
listen 8080;                           # Change port
root "D:/projects/another-app";        # Change project
client_max_body_size 50m;              # File upload limit
```

## Stopping

```bash
futou stop nginx
```

## PHP Integration (FastCGI)

To proxy PHP requests to a FastCGI server:

```nginx
location ~ \.php$ {
    fastcgi_pass 127.0.0.1:9000;
    fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
    include fastcgi_params;
}
```

Then start PHP as a FastCGI server:

```bash
php-cgi.exe -b 127.0.0.1:9000
```
