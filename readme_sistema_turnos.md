# Sistema de Turnos - Grupo 02

## Definición de Entornos de Desarrollo

Este documento describe cómo configurar y ejecutar el entorno de desarrollo del proyecto de forma local, asegurando que cualquier integrante del equipo pueda reproducir el sistema en su máquina.

---

## 🚀 Quick Start (ejecución rápida)

Clonar el repositorio:

```bash
git clone git@gitlab.com:ayudantes-ingsoft/sistema-turnos-grupo-02.git
cd sistema-turnos-grupo-02
```

Levantar el sistema:

```bash
docker compose up -d --build --remove-orphans
```

Abrir en navegador:

http://localhost:20000

---

## 🧩 Requisitos

- Docker (versión reciente)
- Git

### Verificación de Docker Compose

El proyecto requiere Docker Compose v2 (`docker compose`).

Verificar instalación:

```bash
docker compose version
```

---

## ⚠️ Docker Compose (detalle importante)

En algunos entornos puede ocurrir que:

- `docker compose` no esté disponible
- o solo esté instalado `docker-compose` (versión antigua)

En ese caso, será necesario instalar o habilitar Docker Compose v2 según el sistema operativo.

---

## 🔐 Configuración de acceso (SSH)

Para clonar el repositorio es necesario contar con acceso mediante SSH a GitLab.

### 1. Generar una clave SSH

```bash
ssh-keygen -t ed25519 -C "tu_email@example.com"
```

### 2. Agregar la clave a GitLab

```bash
cat ~/.ssh/id_ed25519.pub
```

Copiar en:

GitLab → User Settings → Access → SSH keys

### 3. Verificar conexión

```bash
ssh -T git@gitlab.com
```

---

## ⚙️ Variables de entorno

El proyecto utiliza un archivo `.env` con valores por defecto.

Variables principales:

- INGRESS_PORT=20000
- DB_PORT=20001
- DB_APP_PASSWORD=dev-password

---

## 🌐 Acceso al sistema

- Aplicación web:
  http://localhost:20000

- API backend:
  http://localhost:20000/api

---

## ✅ Validación del entorno

El entorno se considera correctamente configurado si:

- `docker compose ps` muestra servicios activos
- La aplicación carga en el navegador
- Se redirige a `/signup`

---

## 🛠 Tecnologías utilizadas

- Java (backend) 
- React (frontend). Según template.
- PostgreSQL (base de datos)
- Docker / Docker Compose
- GitLab (CI/CD)

---

## 🐞 Problemas comunes

- Docker Compose no disponible

  Verificar:

  ```bash
  docker compose version
  ```

- Puertos ocupados

  Si los puertos 20000 o 20001 están en uso, se pueden modificar en el archivo `.env`.

---

## 🧠 Justificación del entorno

El uso de Docker permite:

- Consistencia entre entornos
- Replicabilidad
- Aislamiento de servicios

---

## 🧾 Conclusión

El entorno permite ejecutar el sistema completo de manera reproducible mediante Docker Compose.
