# 🚀 Nexus CMS

> A modular Headless CMS built with Rust (Axum) + React

![Rust](https://img.shields.io/badge/Rust-1.75+-dea584?style=flat&logo=rust)
![React](https://img.shields.io/badge/React-18+-61dafb?style=flat&logo=react)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15+-336791?style=flat&logo=postgresql)
![License](https://img.shields.io/badge/License-MIT-green.svg)

Nexus is a powerful, modular headless CMS that separates the content management backend from the frontend presentation layer. Built with Rust for high performance and React for a modern user experience.

![Nexus CMS Dashboard](./docs/screenshots/dashboard.png)

## ✨ Features

### Backend (Rust/Axum)
- 🔐 **JWT Authentication** - Secure token-based auth with refresh tokens
- 📝 **Page Management** - Create, edit, publish pages with SEO metadata
- 🧱 **Block System** - Modular content blocks (Hero, RichText, ProjectGrid, etc.)
- 📁 **Collections** - Custom post types with dynamic schemas
- 🖼️ **Media Management** - File uploads with drag & drop
- 👥 **Role-Based Access** - Granular permissions (Super-Architecte, Gestionnaire, VIP, Visiteur)
- ⚡ **Rate Limiting** - Built-in DDoS protection
- 🛡️ **Security Headers** - CORS, CSP, HSTS, X-Frame-Options
- 🔧 **Maintenance Mode** - System-wide maintenance with admin bypass

### Frontend (React/Vite)
- 🎨 **Modern UI** - Clean, responsive design
- 🔄 **Real-time Preview** - Live block preview while editing
- 🧭 **Intuitive Navigation** - Sidebar, tabs, and breadcrumbs
- 📱 **Responsive** - Works on desktop and mobile
- 🔐 **Protected Routes** - Auth-gated admin areas

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (React)                      │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │  Pages  │  │  Admin  │  │  Auth   │  │  Block Ed.  │  │
│  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬──────┘  │
└───────┼────────────┼────────────┼──────────────┼──────────┘
        │            │            │              │
        └────────────┴────────────┴──────────────┘
                           │
                    REST API (/api/v1)
                           │
┌───────────────────────────┼───────────────────────────────┐
│                     Backend (Rust)                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌───────────┐ │
│  │ Auth    │  │ Pages   │  │ Blocks  │  │  Media    │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └─────┬─────┘ │
│       └────────────┴────────────┴──────────────┘       │
│                           │                              │
│                    PostgreSQL                             │
└───────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- Node.js 18+ (for frontend development)
- Rust 1.75+ (for backend development)

### Using Docker Compose

```bash
# Clone the repository
git clone https://github.com/TestD-Aho/nexus.git
cd nexus

# Start all services
docker-compose up -d

# Access the application
Frontend:  http://localhost:5173
Backend:   http://localhost:3000
```

### Manual Setup

#### Backend
```bash
cd backend

# Create environment file
cp .env.example .env
# Edit .env with your database URL and JWT secret

# Run migrations and start server
cargo run
```

#### Frontend
```bash
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev
```

## 📡 API Endpoints

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| `POST` | `/api/v1/auth/login` | User login | ❌ |
| `POST` | `/api/v1/auth/register` | User registration | ❌ |
| `GET` | `/api/v1/auth/me` | Get current user | ✅ |
| `GET` | `/api/v1/pages` | List all pages | ❌ |
| `POST` | `/api/v1/pages` | Create page | ✅ |
| `GET` | `/api/v1/pages/:slug` | Get page by slug | ❌ |
| `PUT` | `/api/v1/pages/:id` | Update page | ✅ |
| `DELETE` | `/api/v1/pages/:id` | Delete page | ✅ |
| `GET` | `/api/v1/blocks` | List blocks | ❌ |
| `POST` | `/api/v1/blocks` | Create block | ✅ |
| `POST` | `/api/v1/blocks/reorder` | Reorder blocks | ✅ |
| `GET` | `/api/v1/media` | List media | ❌ |
| `POST` | `/api/v1/media/upload` | Upload file | ✅ |
| `GET` | `/api/v1/admin/stats` | Admin statistics | 🔒 |
| `GET` | `/api/v1/admin/users` | List users | 🔒 |
| `PUT` | `/api/v1/system/maintenance` | Toggle maintenance | 🔒 |

## 🧱 Block Types

| Block Type | Description |
|------------|-------------|
| `HeroHeader` | Full-width hero section with title, subtitle, background |
| `RichText` | HTML rich text content |
| `ProjectGrid` | Grid display of projects/portfolio items |
| `SkillMatrix` | Skills/tags display |
| `ContactForm` | Contact form with validation |
| `TestimonialSlider` | Client testimonials/quotes |

## 👥 Roles & Permissions

| Role | Description | Permissions |
|------|-------------|-------------|
| `Super-Architecte` | Root admin | Full system access |
| `Gestionnaire` | Content manager | Pages, blocks, media, collections |
| `VIP` | Premium user | Read public content |
| `Visiteur` | Anonymous visitor | Read public content |

## 🖥️ Screenshots

### Login Page
![Login](./docs/screenshots/login.png)

### Admin Dashboard
![Dashboard](./docs/screenshots/dashboard.png)

### Page Editor with Block Builder
![Page Editor](./docs/screenshots/page-editor.png)

### Media Library
![Media Library](./docs/screenshots/media-library.png)

### Public Page
![Public Page](./docs/screenshots/public-page.png)

## 🛠️ Tech Stack

### Backend
- **Runtime**: Rust
- **Framework**: Axum
- **Database**: PostgreSQL
- **ORM**: SQLx
- **Auth**: JWT (Argon2 hashing)
- **Validation**: serde

### Frontend
- **Framework**: React 18
- **Build Tool**: Vite
- **Routing**: React Router 6
- **HTTP Client**: Axios
- **Styling**: Custom CSS

### Infrastructure
- **Container**: Docker
- **Orchestration**: Docker Compose

## 📁 Project Structure

```
nexus/
├── backend/
│   ├── src/
│   │   ├── api/          # API routes
│   │   ├── db/           # Database & migrations
│   │   ├── middleware/   # Security & rate limiting
│   │   ├── models/       # Data models
│   │   ├── services/     # Business logic
│   │   └── utils/        # Helpers
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/
│   ├── src/
│   │   ├── api/          # API client
│   │   ├── components/   # Reusable components
│   │   ├── context/      # React context
│   │   ├── pages/        # Page components
│   │   └── App.jsx       # Main app
│   ├── package.json
│   └── Dockerfile
├── docs/
│   └── screenshots/      # README images
├── docker-compose.yml
└── README.md
```

## 🔧 Configuration

### Environment Variables (Backend)

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `JWT_SECRET` | Secret key for JWT signing | Required |
| `NEXUS_HOST` | Server host | `0.0.0.0` |
| `NEXUS_PORT` | Server port | `3000` |
| `NEXUS_UPLOAD_DIR` | Upload directory | `./uploads` |
| `NEXUS_MAX_UPLOAD_SIZE` | Max upload size (bytes) | `10485760` |

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

<p align="center">Built with ❤️ using Rust + React</p>
