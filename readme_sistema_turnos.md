# 🛠️ Definición del Entorno de Desarrollo

Este documento describe cómo configurar y ejecutar el entorno de desarrollo del proyecto de forma local, asegurando que cualquier integrante del equipo pueda reproducir el sistema en su computadora.

---

## 🚀 Quick Start

Clonar el repositorio proveído por la cátedra:

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

- Git
- Docker (versión reciente)

### Verificación de Docker Compose

El proyecto requiere Docker Compose v2 (`docker compose`).

Verificar instalación con:

```bash
docker compose version
```

---

## 🔐 Configuración de acceso (SSH)

Para clonar el repositorio es necesario contar con acceso mediante SSH a GitLab.

### 1. Generar una clave SSH

```bash
ssh-keygen -t ed25519 -C "tu_email@example.com"
```

⚠️ Importante:

Si el archivo `~/.ssh/id_ed25519` ya existe (por ejemplo, porque se utiliza para GitHub u otro servicio), el sistema mostrará un mensaje indicando que el archivo ya existe.

En ese caso, no sobrescribir la clave existente. En su lugar, especificar un nuevo nombre, por ejemplo:


`/home/usuario/.ssh/id_ed25519_gitlab`

De esta forma se evita interferir con otras configuraciones SSH.

### 2. Agregar la clave a GitLab

```bash
cat ~/.ssh/id_ed25519.pub
```
(Si se utilizó otro nombre, ajustar el comando correspondiente)

Copiar en:

GitLab → User Settings → Access → SSH keys

### 3. Registrar la clave en el agente SSH

Si se creó una nueva clave, es necesario cargarla en el agente SSH:

```bash
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519
```

### 4. Verificar conexión

```bash
ssh -T git@gitlab.com
```

Si la configuración es correcta, se mostrará un mensaje de bienvenida indicando que la autenticación fue exitosa.

---

## 🐳 Docker Compose (detalle importante)

El proyecto utiliza **Docker Compose v2**, el cual se ejecuta mediante el comando:

`docker compose`

(no confundir con `docker-compose`, que corresponde a una versión anterior)

---
### Posibles problemas

En algunos entornos puede ocurrir que:

- El comando `docker compose` no esté disponible  
- Solo funcione `docker-compose` (versión antigua)  
- Se obtenga un error como:

  `unknown command: docker compose`

### Verificación

Para verificar si Docker Compose v2 está correctamente instalado:
```bash
docker compose version
```


---

### Solución

Si el comando no está disponible, es necesario instalar o habilitar Docker Compose v2.

Esto puede implicar:

- Instalar el plugin de Docker Compose  
- Actualizar la instalación de Docker  
- Configurar correctamente el entorno (por ejemplo en WSL)

⚠️ En este proyecto **no es suficiente usar `docker-compose`**, ya que el archivo `docker-compose.yml` utiliza características de la versión 2.

### Recomendación

Se recomienda utilizar una instalación reciente de Docker que incluya soporte nativo para `docker compose`, para evitar problemas de compatibilidad.

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

- Base de datos (PostgreSQL): http://localhost:20001

  Nota: este puerto no es accesible desde el navegador, sino mediante clientes de base de datos.

---

## ✅ Validación del entorno

El entorno se considera correctamente configurado y listo para comenzar a trabajar si:

- `docker compose ps` muestra servicios activos
- La aplicación carga en el navegador
- Se redirige a `/signup`

---

## 🛠 Tecnologías utilizadas

- Java (backend) 
- React (frontend). Según template.
- PostgreSQL (base de datos). Según template.
- Docker / Docker Compose
- GitLab (CI/CD)

---

## 🐞 Otros problemas comunes

- Puertos ocupados

  Si los puertos 20000 o 20001 están en uso, se pueden modificar en el archivo `.env`.

---

🧾 Conclusión

Se definió y validó un entorno de desarrollo reproducible utilizando Docker y Docker Compose.

Se logró:

- Clonar el repositorio mediante acceso SSH
- Levantar el sistema completo utilizando Docker Compose
- Acceder a la aplicación desde el navegador en http://localhost:20000
- Verificar el correcto funcionamiento de los servicios mediante `docker compose ps`
- Confirmar la inicialización del backend y la conexión a la base de datos a través de los logs

El entorno permite ejecutar el sistema de manera consistente en distintas máquinas, facilitando el desarrollo y evitando problemas de configuración local.