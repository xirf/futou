# Apache

Apache HTTP Server from [Apache Lounge](https://www.apachelounge.com/download/).

## Versions

| Version | Source |
|---------|--------|
| 2.4.66 | VS17, x64 |

::: info Apache Lounge
Apache Lounge provides up-to-date Windows binaries with the latest Visual Studio compiler. These are referenced by the ASF, Microsoft, and PHP.
:::

## Starting

```bash
futou start apache 2.4.66
```

When starting from the GUI, a document root prompt appears. Enter the path to your project directory. The daemon generates `httpd.conf` and launches Apache:

```
DocumentRoot "D:/projects/myapp"
Listen 80
```

The generated config is saved to `%APPDATA%\.futou\runtimes\apache\<version>\data\conf\httpd.conf`. **It is only generated on first start** — subsequent starts preserve your edits.

## Configuration

The GUI Config button opens the generated `httpd.conf`. You can customize:

```apache
Listen 8080                          # Change port
DocumentRoot "D:/projects/myapp"     # Change project
LoadModule rewrite_module modules/mod_rewrite.so  # Enable rewrite
```

## Stopping

```bash
futou stop apache
```

## PHP Integration

To use PHP with Apache, add to the config:

```apache
LoadModule proxy_module modules/mod_proxy.so
LoadModule proxy_fcgi_module modules/mod_proxy_fcgi.so

<FilesMatch \.php$>
    SetHandler proxy:fcgi://127.0.0.1:9000
</FilesMatch>
```

Then start PHP as a FastCGI server using the activated PHP version:

```bash
php-cgi.exe -b 127.0.0.1:9000
```
