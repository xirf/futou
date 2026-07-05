# PHP

PHP binaries from [windows.php.net](https://windows.php.net/downloads/releases/).

## Versions

| Version | Source |
|---------|--------|
| 8.4.23 | NTS, VS17, x64 |
| 8.3.32 | NTS, VS16, x64 |
| 8.2.32 | NTS, VS16, x64 |

All builds are **Non-Thread-Safe (NTS)** — suitable for CLI usage and FastCGI with Apache/Nginx.

## Activation

```bash
futou use php 8.4.23
```

Creates `.bat` shims for all PHP binaries (`php.exe`, `php-cgi.exe`, `phpdbg.exe`).

## Built-in Server

PHP includes a built-in development server:

```bash
php -S localhost:8000
```

No Apache/Nginx needed for quick testing.

## Configuration

The GUI Config button opens `php.ini-development` if it exists in the runtime directory. Copy it to `php.ini` to customize:

```bash
copy php.ini-development php.ini
```

## Apache Integration (Manual)

When Apache generates its config and PHP is installed, you can add PHP-FCGI support manually:

```apache
LoadModule proxy_module modules/mod_proxy.so
LoadModule proxy_fcgi_module modules/mod_proxy_fcgi.so

<FilesMatch \.php$>
    SetHandler proxy:fcgi://127.0.0.1:9000
</FilesMatch>
```

Then start PHP as a FastCGI server:

```bash
php-cgi.exe -b 127.0.0.1:9000
```
