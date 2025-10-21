# Lighttpd Web Server Setup

Complete setup documentation for serving the Quality Control Room UI with lighttpd and HTTPS.

## Overview

- **Web Server**: lighttpd 1.4.79
- **SSL/TLS**: Let's Encrypt certificate via certbot
- **Domain**: quality-control.io
- **Document Root**: `/home/vp/quality_control_room/ui/dist`
- **Ports**: 80 (HTTP, redirects to HTTPS), 443 (HTTPS)

## Installation

### 1. Install lighttpd

```bash
sudo apt install -y lighttpd
```

**Installed packages:**
- lighttpd 1.4.79
- lighttpd-mod-openssl (SSL/TLS support)
- lighttpd-mod-deflate (compression)
- libdeflate0 (compression library)

### 2. Install certbot

```bash
sudo apt install -y certbot
```

**Version:** certbot 4.0.0

## SSL Certificate Setup

### Obtain Certificate

1. Stop lighttpd temporarily:
```bash
sudo systemctl stop lighttpd
```

2. Request certificate from Let's Encrypt:
```bash
sudo certbot certonly --standalone \
  -d quality-control.io \
  --non-interactive \
  --agree-tos \
  --email VasiliiPiiadov@users.noreply.github.com
```

**Certificate locations:**
- Fullchain: `/etc/letsencrypt/live/quality-control.io/fullchain.pem`
- Private key: `/etc/letsencrypt/live/quality-control.io/privkey.pem`
- Chain: `/etc/letsencrypt/live/quality-control.io/chain.pem`
- **Expires:** January 18, 2026
- **Auto-renewal:** Enabled via certbot.timer systemd service

### Create Combined PEM File

Lighttpd requires the private key and certificate in a single file:

```bash
sudo bash -c 'cat /etc/letsencrypt/live/quality-control.io/privkey.pem \
  /etc/letsencrypt/live/quality-control.io/fullchain.pem \
  > /etc/letsencrypt/live/quality-control.io/combined.pem'
```

## Lighttpd Configuration

### Main Configuration

File: `/etc/lighttpd/lighttpd.conf`

**Modified settings:**
```conf
server.document-root = "/home/vp/quality_control_room/ui/dist"
```

### Custom Configuration

File: `/etc/lighttpd/conf-available/99-quality-control.conf`

```conf
# Quality Control Room Configuration

# Enable required modules (must come before other settings)
server.modules += (
    "mod_openssl",
    "mod_deflate",
    "mod_setenv",
    "mod_rewrite"
)

# SSL Configuration for HTTPS (port 443)
$SERVER["socket"] == ":443" {
    ssl.engine = "enable"
    ssl.pemfile = "/etc/letsencrypt/live/quality-control.io/combined.pem"
    ssl.ca-file = "/etc/letsencrypt/live/quality-control.io/chain.pem"
    
    # Modern SSL/TLS settings
    ssl.openssl.ssl-conf-cmd = ("MinProtocol" => "TLSv1.2")
    ssl.cipher-list = "ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384"
    ssl.honor-cipher-order = "disable"
}

# HTTP to HTTPS redirect
$SERVER["socket"] == ":80" {
    $HTTP["host"] =~ "quality-control\.io" {
        url.redirect = ("^/(.*)" => "https://quality-control.io/$1")
    }
}

# Compression for static assets
deflate.mimetypes = (
    "text/html",
    "text/plain",
    "text/css",
    "text/javascript",
    "application/javascript",
    "application/json"
)

# SPA routing - serve index.html for all routes
url.rewrite-once = (
    "^/(css|js|assets|img|fonts)/.*" => "$0",
    "^/(favicon\.ico|robots\.txt)$" => "$0",
    "^/.*\.(css|js|json|svg|png|jpg|jpeg|gif|ico|woff|woff2|ttf|eot)$" => "$0",
    "^/(.*)$" => "/index.html"
)

# Cache control headers
$HTTP["url"] =~ "\.(css|js|jpg|png|svg|woff|woff2)$" {
    setenv.add-response-header = ( "Cache-Control" => "public, max-age=31536000" )
}
```

### Enable Custom Configuration

```bash
sudo ln -sf /etc/lighttpd/conf-available/99-quality-control.conf \
  /etc/lighttpd/conf-enabled/99-quality-control.conf
```

### Fix Permissions

Allow the web server (www-data user) to access the UI files:

```bash
# Allow access to home directory
chmod o+rx /home/vp

# Allow access to dist directory
chmod -R o+rX /home/vp/quality_control_room/ui/dist
```

## Service Management

### Test Configuration

```bash
sudo lighttpd -t -f /etc/lighttpd/lighttpd.conf
```

Expected output: `Syntax OK`

### Start/Stop/Restart

```bash
# Start
sudo systemctl start lighttpd

# Stop
sudo systemctl stop lighttpd

# Restart
sudo systemctl restart lighttpd

# Status
sudo systemctl status lighttpd
```

### Enable Auto-start

```bash
sudo systemctl enable lighttpd
```

### View Logs

```bash
# Error log
sudo tail -f /var/log/lighttpd/error.log

# Access log (if enabled)
sudo tail -f /var/log/lighttpd/access.log

# Systemd journal
sudo journalctl -u lighttpd -f
```

## Testing

### Check Listening Ports

```bash
sudo ss -tlnp | grep lighttpd
```

Expected output:
```
LISTEN 0 4096 0.0.0.0:443 0.0.0.0:* users:(("lighttpd",pid=XXXXX,fd=4))
LISTEN 0 4096 0.0.0.0:80  0.0.0.0:* users:(("lighttpd",pid=XXXXX,fd=6))
LISTEN 0 4096    [::]:80     [::]:* users:(("lighttpd",pid=XXXXX,fd=5))
```

### Test HTTPS

```bash
curl -I https://quality-control.io/
```

Expected output:
```
HTTP/2 200
content-type: text/html
...
```

### Test HTTP Redirect

```bash
curl -I http://quality-control.io/
```

Expected output:
```
HTTP/1.1 301 Moved Permanently
Location: https://quality-control.io/index.html
...
```

### Test in Browser

Open: https://quality-control.io/

## Certificate Renewal

### Automatic Renewal

Certbot automatically renews certificates via systemd timer:

```bash
# Check timer status
sudo systemctl status certbot.timer

# View renewal schedule
sudo systemctl list-timers certbot.timer
```

### Manual Renewal

```bash
# Stop lighttpd
sudo systemctl stop lighttpd

# Renew certificate
sudo certbot renew

# Recreate combined PEM file
sudo bash -c 'cat /etc/letsencrypt/live/quality-control.io/privkey.pem \
  /etc/letsencrypt/live/quality-control.io/fullchain.pem \
  > /etc/letsencrypt/live/quality-control.io/combined.pem'

# Start lighttpd
sudo systemctl start lighttpd
```

### Post-Renewal Hook

To automatically recreate the combined PEM file and reload lighttpd after renewal, create:

File: `/etc/letsencrypt/renewal-hooks/post/lighttpd-reload.sh`

```bash
#!/bin/bash
# Recreate combined PEM
cat /etc/letsencrypt/live/quality-control.io/privkey.pem \
    /etc/letsencrypt/live/quality-control.io/fullchain.pem \
    > /etc/letsencrypt/live/quality-control.io/combined.pem

# Reload lighttpd
systemctl reload lighttpd
```

Make it executable:
```bash
sudo chmod +x /etc/letsencrypt/renewal-hooks/post/lighttpd-reload.sh
```

## Updating the UI

After rebuilding the UI:

```bash
# Navigate to UI directory
cd /home/vp/quality_control_room/ui

# Rebuild
npm run build

# Ensure permissions (if needed)
chmod -R o+rX dist

# Lighttpd will automatically serve the new files
```

No need to restart lighttpd - it serves static files directly.

## Troubleshooting

### 403 Forbidden Error

**Problem:** Cannot access the website, getting 403 error

**Solution:** Check file permissions
```bash
# Allow access to home directory
chmod o+rx /home/vp

# Allow access to dist directory
chmod -R o+rX /home/vp/quality_control_room/ui/dist

# Check ownership
ls -la /home/vp/quality_control_room/ui/dist
```

### SSL Certificate Errors

**Problem:** Browser shows SSL warning

**Solution:** 
1. Check certificate files exist:
```bash
ls -l /etc/letsencrypt/live/quality-control.io/
```

2. Verify combined.pem was created:
```bash
ls -l /etc/letsencrypt/live/quality-control.io/combined.pem
```

3. Check lighttpd error log:
```bash
sudo tail -100 /var/log/lighttpd/error.log
```

### Port Already in Use

**Problem:** lighttpd fails to start, port already in use

**Solution:** Find and stop the conflicting service
```bash
# Check what's using port 80
sudo lsof -i :80

# Check what's using port 443
sudo lsof -i :443

# Stop conflicting service (example: nginx)
sudo systemctl stop nginx
```

### Configuration Syntax Error

**Problem:** lighttpd fails to start after config changes

**Solution:** Test configuration
```bash
sudo lighttpd -t -f /etc/lighttpd/lighttpd.conf
```

Check the error message and fix the configuration file.

### SPA Routes Return 404

**Problem:** Vue.js routes work on homepage but return 404 when accessed directly

**Solution:** This should be handled by the `url.rewrite-once` rules in the config. If not working:

1. Verify rewrite module is enabled in the configuration
2. Check lighttpd error log for rewrite rule issues
3. Ensure the rewrite rules are in the correct order

## Security Considerations

1. **TLS Version:** Minimum TLSv1.2 enforced
2. **Cipher Suites:** Modern, secure cipher suites configured
3. **File Permissions:** Only necessary files are world-readable
4. **Auto-updates:** Certbot auto-renewal ensures certificates stay valid
5. **HTTP Redirect:** All HTTP traffic redirected to HTTPS

## Performance Optimizations

1. **Compression:** Gzip compression enabled for text files
2. **Cache Headers:** Long cache times for static assets (1 year)
3. **HTTP/2:** Automatically enabled with SSL/TLS

## Firewall Configuration

If using a firewall (ufw, iptables), ensure ports 80 and 443 are open:

```bash
# Using ufw
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Using iptables
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
```

## Summary

✅ **Web server:** lighttpd 1.4.79  
✅ **SSL/TLS:** Let's Encrypt certificate (expires 2026-01-18)  
✅ **Auto-renewal:** Enabled via certbot.timer  
✅ **HTTP → HTTPS:** Automatic redirect configured  
✅ **SPA routing:** Vue Router compatible  
✅ **Compression:** Enabled for text content  
✅ **Caching:** Long-term cache for static assets  
✅ **Security:** Modern TLS configuration  

**Access:** https://quality-control.io/

---

**Last updated:** October 20, 2025  
**Status:** ✅ Production ready
