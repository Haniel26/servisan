# Product Requirement Document (PRD)

## 1. Project Overview

* **Project Name:** Servisan

* **Objective:** Lightweight, fast, and portable local development tool tailored for PHP and MySQL/MariaDB environments, serving as a minimal, high-performance alternative to Laragon.

* **Core Language:** Rust

---

## 2. Naming Rationale: Why "Servisan"?

The name **Servisan** was chosen for its rich combination of developer terminology and clever cultural wordplay:

1. **Server + Artisan:** Combines the core functionality of running a local **Server** with the elegance of Laravel's **Artisan** CLI ecosystem.
2. **"Serve" Action:** Directly references the iconic `php artisan serve` command that PHP developers run daily.
3. **Unique Wordplay:** In Indonesian, *"servisan"* colloquially means a tuned-up or repaired mechanism—symbolizing a fine-tuned, reliable, and high-performance development environment.
4. **Distinct Identity:** Unique on search engines and GitHub, providing a memorable brand without being tied strictly to a single language keyword.

---

## 3. Key Architecture & Dependencies Strategy

* **Binary Reuse Strategy:** Utilize existing portable PHP and MySQL/MariaDB binaries from an existing Laragon installation (`laragon/bin/php/...` & `laragon/bin/mysql/...`).

* **Isolation Principle:** Entire application, including configuration files, environment variables, and databases, must be self-contained in a single directory. Zero installation overhead, zero dirty Windows Registry entries.

---

## 4. Recommended Directory Structure

```
servisan/
├── bin/
│   ├── php/           <-- Copied directly from Laragon (e.g. php-8.x.x-Win32-...)
│   └── mysql/         <-- Copied directly from Laragon (e.g. mysql-8.x.x-winx64)
├── www/               <-- Web root directory for user projects
├── data/              <-- MySQL database storage directory
├── config/            <-- App settings (app.json or app.toml)
└── servisan.exe       <-- Built Rust application binary
```

---

## 5. Functional Requirements

### Phase 1: MVP (CLI / Core Engine)

1. **Service Lifecycle Management:**

   * **Start All Services:** Spawns `mysqld.exe` and `php-cgi.exe` / `php.exe` (via Built-in Web Server or Router).

   * **Stop All Services:** Gracefully sends termination signals (`SIGTERM` / `KILL`) to spawned child processes.

   * **Process Monitoring:** Periodically check process status to verify if MySQL/PHP are running or unexpectedly crashed.

2. **PHP Engine Options (Pick One for MVP):**

   * **Option A (Simplest):** PHP Built-in Web Server (`php -S 127.0.0.1:8000 -t www`).

   * **Option B (Laragon Style):** Lightweight Nginx embedded + `php-cgi.exe`.

3. **MySQL Data Directory Initialization:**

   * Automatically target `data/` folder for MySQL storing using `--datadir` argument.

4. **Environment Path Injection:**

   * Temporarily inject PHP and MySQL `bin` directory paths into the application process runtime `PATH` so tools like `composer` or `mysql` CLI are accessible.

### Phase 2: Enhanced Features

1. **Automatic VirtualHost / Local Domain:**

   * Watch `www/` directory for subfolders (e.g., `www/mysite`).

   * Append domain mapping to `C:\Windows\System32\drivers\etc\hosts` (e.g., `127.0.0.1 mysite.test`). *Requires Admin privileges.*

2. **Tray Icon System:**

   * Minimize Rust app to Windows System Tray with Right-Click Context Menu (Start/Stop, Open `www`, Exit).

---

## 6. Non-Functional Requirements

* **Performance:** Startup latency under 100 milliseconds. Memory footprint < 15MB for the manager executable.

* **Portability:** Moving `servisan/` folder to another drive/PC should work without re-configuration.

* **Error Handling:** Robust handling for port conflicts (e.g., if port 80/8000 or 3306 is already occupied).

---

## 7. Suggested Rust Crates Ecosystem

* **`tokio` / `std::process::Command`**: For async or standard process management (spawning `mysqld` / `php`).

* **`sysinfo`**: For process detection and PID tracking.

* **`ctrlc`**: To gracefully trap `Ctrl+C` in CLI mode and shut down spawned MySQL/PHP child processes.

* **`tray-icon` / `tao`** *(For Phase 2)*: For creating a lightweight Windows system tray application without heavy webviews.

* **`serde` + `toml`**: For parsing app configurations.

---

## 8. Implementation Steps (Action Plan)

1. **Setup Folder Structure:**
   Create project folder and copy PHP and MySQL binaries into `bin/php` and `bin/mysql`.

2. **Initialize Rust Project:**
   `cargo new servisan`

3. **Draft Process Spawner:**
   Write code to execute `bin/mysql/bin/mysqld.exe` with arguments and `bin/php/php.exe -S 127.0.0.1:8000 -t www`.

4. **Implement Graceful Shutdown:**
   Ensure process cleanup on close so `mysqld.exe` doesn't leak into Task Manager.

5. **Testing & Refinement.**