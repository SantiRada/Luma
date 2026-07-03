CREATE TABLE IF NOT EXISTS users (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(120) NOT NULL,
    email VARCHAR(190) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS password_resets (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id INT UNSIGNED NOT NULL,
    token_hash CHAR(64) NOT NULL,
    expires_at DATETIME NOT NULL,
    used_at DATETIME NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX (token_hash),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS actions (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id INT UNSIGNED NOT NULL,
    manifest_id VARCHAR(190) NOT NULL,
    slug VARCHAR(190) NOT NULL UNIQUE,
    name VARCHAR(160) NOT NULL,
    version VARCHAR(40) NOT NULL,
    short_description VARCHAR(280) NOT NULL,
    description TEXT NULL,
    category ENUM('utility','design','dev','productivity','other') NOT NULL DEFAULT 'other',
    tags VARCHAR(255) NULL,
    file_path VARCHAR(255) NOT NULL,
    file_size INT UNSIGNED NOT NULL DEFAULT 0,
    downloads INT UNSIGNED NOT NULL DEFAULT 0,
    is_public TINYINT(1) NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY manifest_version_unique (manifest_id, version),
    INDEX (category),
    INDEX (user_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS saved_actions (
    user_id INT UNSIGNED NOT NULL,
    action_id INT UNSIGNED NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, action_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (action_id) REFERENCES actions(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS reports (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id INT UNSIGNED NULL,
    title VARCHAR(180) NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS product_versions (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    version VARCHAR(40) NOT NULL UNIQUE,
    changelog TEXT NOT NULL,
    file_path VARCHAR(255) NOT NULL,
    file_size BIGINT UNSIGNED NOT NULL DEFAULT 0,
    is_current TINYINT(1) NOT NULL DEFAULT 0,
    downloads INT UNSIGNED NOT NULL DEFAULT 0,
    created_by INT UNSIGNED NULL,
    published_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX (is_current),
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL
) ENGINE=InnoDB;

INSERT INTO product_versions (version, changelog, file_path, file_size, is_current)
VALUES (
    '0.1.0',
    'Primera version publica de LUMA. Incluye launcher de escritorio, sistema de Actions, instalacion de paquetes .lm, componentes base, soporte para Translate Image, Translate This, ContarCaracteres, Merge PDF, Image Convert y Video Downloader, tray en segundo plano e inicio automatico con Windows. Las Actions se instalan por separado.',
    'versions/LUMA_0.1.0.exe',
    230156237,
    1
)
ON DUPLICATE KEY UPDATE
    changelog = VALUES(changelog),
    file_path = VALUES(file_path),
    file_size = VALUES(file_size),
    is_current = VALUES(is_current);
