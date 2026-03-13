// Layout Component
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

export function Layout({ children }) {
  const { user, logout, isAdmin } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="app-layout">
      <nav className="navbar">
        <div className="nav-brand">
          <Link to="/">Nexus CMS</Link>
        </div>
        <div className="nav-links">
          <Link to="/">Pages</Link>
          {isAdmin && <Link to="/admin">Admin</Link>}
          {user ? (
            <button onClick={handleLogout} className="btn-logout">
              Logout
            </button>
          ) : (
            <Link to="/login">Login</Link>
          )}
        </div>
      </nav>
      <main className="main-content">
        {children}
      </main>
    </div>
  );
}
