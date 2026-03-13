import { useState, useEffect } from 'react'
import { BrowserRouter, Routes, Route, Link } from 'react-router-dom'
import axios from 'axios'

const API_URL = 'http://localhost:3000/api/v1'

function App() {
  return (
    <BrowserRouter>
      <div className="app">
        <nav className="navbar">
          <div className="nav-brand">Nexus CMS</div>
          <div className="nav-links">
            <Link to="/">Pages</Link>
            <Link to="/admin">Admin</Link>
          </div>
        </nav>
        <main className="main-content">
          <Routes>
            <Route path="/" element={<PagesList />} />
            <Route path="/page/:slug" element={<PageView />} />
            <Route path="/admin" element={<Admin />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  )
}

function PagesList() {
  const [pages, setPages] = useState([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    axios.get(`${API_URL}/pages`)
      .then(res => {
        setPages(res.data)
        setLoading(false)
      })
      .catch(err => {
        console.error(err)
        setLoading(false)
      })
  }, [])

  if (loading) return <div className="loading">Loading...</div>

  return (
    <div className="pages-list">
      <h1>Pages</h1>
      <div className="pages-grid">
        {pages.map(page => (
          <Link key={page.id} to={`/page/${page.slug}`} className="page-card">
            <h3>{page.title}</h3>
            <span className="page-slug">/{page.slug}</span>
            <span className={`status ${page.is_published ? 'published' : 'draft'}`}>
              {page.is_published ? 'Published' : 'Draft'}
            </span>
          </Link>
        ))}
      </div>
    </div>
  )
}

function PageView() {
  const [page, setPage] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  // Get slug from URL
  const slug = window.location.pathname.split('/').pop()

  useEffect(() => {
    axios.get(`${API_URL}/pages/${slug}`)
      .then(res => {
        setPage(res.data)
        setLoading(false)
      })
      .catch(err => {
        setError(err.response?.status === 404 ? 'Page not found' : 'Error loading page')
        setLoading(false)
      })
  }, [slug])

  if (loading) return <div className="loading">Loading...</div>
  if (error) return <div className="error">{error}</div>

  return (
    <div className="page-view">
      <h1>{page?.page?.title}</h1>
      <p className="description">{page?.page?.description}</p>
      
      <div className="blocks">
        {page?.blocks?.map((block, index) => (
          <div key={index} className={`block block-${block.type?.toLowerCase()}`}>
            {block.type === 'HeroHeader' && (
              <div className="hero">
                <h2>{block.title || block.content?.title}</h2>
                <p>{block.content?.subtitle}</p>
              </div>
            )}
            {block.type === 'RichText' && (
              <div className="rich-text" dangerouslySetInnerHTML={{ __html: block.content?.html }} />
            )}
            {block.type === 'ProjectGrid' && (
              <div className="project-grid">
                <h3>{block.content?.title || 'Projects'}</h3>
                <div className="projects">
                  {block.content?.items?.map((item, i) => (
                    <div key={i} className="project-card">
                      <h4>{item.title}</h4>
                      <p>{item.description}</p>
                    </div>
                  ))}
                </div>
              </div>
            )}
            {block.type === 'SkillMatrix' && (
              <div className="skills">
                <h3>Skills</h3>
                <div className="skill-list">
                  {block.content?.skills?.map((skill, i) => (
                    <span key={i} className="skill-tag">{skill}</span>
                  ))}
                </div>
              </div>
            )}
            {block.type === 'ContactForm' && (
              <form className="contact-form" onSubmit={e => e.preventDefault()}>
                <h3>Contact</h3>
                <input type="text" placeholder="Name" />
                <input type="email" placeholder="Email" />
                <textarea placeholder="Message" />
                <button type="submit">Send</button>
              </form>
            )}
            {block.type === 'TestimonialSlider' && (
              <div className="testimonials">
                <h3>Testimonials</h3>
                {block.content?.testimonials?.map((t, i) => (
                  <blockquote key={i}>{t.text} — {t.author}</blockquote>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}

function Admin() {
  const [token, setToken] = useState(localStorage.getItem('nexus_token'))
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [pages, setPages] = useState([])
  const [newPage, setNewPage] = useState({ slug: '', title: '', description: '' })
  const [error, setError] = useState(null)

  const login = async (e) => {
    e.preventDefault()
    try {
      const res = await axios.post(`${API_URL}/auth/login`, { email, password })
      setToken(res.data.token)
      localStorage.setItem('nexus_token', res.data.token)
      setError(null)
    } catch (err) {
      setError('Invalid credentials')
    }
  }

  const logout = () => {
    setToken(null)
    localStorage.removeItem('nexus_token')
  }

  const createPage = async (e) => {
    e.preventDefault()
    try {
      await axios.post(`${API_URL}/pages`, newPage, {
        headers: { Authorization: `Bearer ${token}` }
      })
      setNewPage({ slug: '', title: '', description: '' })
      loadPages()
    } catch (err) {
      setError('Failed to create page')
    }
  }

  const loadPages = async () => {
    const res = await axios.get(`${API_URL}/pages`)
    setPages(res.data)
  }

  useEffect(() => {
    if (token) loadPages()
  }, [token])

  if (!token) {
    return (
      <div className="login-form">
        <h1>Admin Login</h1>
        <form onSubmit={login}>
          <input type="email" placeholder="Email" value={email} onChange={e => setEmail(e.target.value)} />
          <input type="password" placeholder="Password" value={password} onChange={e => setPassword(e.target.value)} />
          <button type="submit">Login</button>
        </form>
        {error && <p className="error">{error}</p>}
        <p className="hint">Default: admin@nexus.local / admin123</p>
      </div>
    )
  }

  return (
    <div className="admin">
      <div className="admin-header">
        <h1>Admin Dashboard</h1>
        <button onClick={logout}>Logout</button>
      </div>
      
      <section className="create-page">
        <h2>Create New Page</h2>
        <form onSubmit={createPage}>
          <input 
            type="text" 
            placeholder="Slug" 
            value={newPage.slug} 
            onChange={e => setNewPage({...newPage, slug: e.target.value})} 
          />
          <input 
            type="text" 
            placeholder="Title" 
            value={newPage.title} 
            onChange={e => setNewPage({...newPage, title: e.target.value})} 
          />
          <input 
            type="text" 
            placeholder="Description" 
            value={newPage.description} 
            onChange={e => setNewPage({...newPage, description: e.target.value})} 
          />
          <button type="submit">Create Page</button>
        </form>
      </section>

      <section className="pages-list">
        <h2>Existing Pages</h2>
        <ul>
          {pages.map(page => (
            <li key={page.id}>
              <span>{page.title}</span>
              <span className="slug">/{page.slug}</span>
            </li>
          ))}
        </ul>
      </section>
    </div>
  )
}

export default App
