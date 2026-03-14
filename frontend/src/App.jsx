// Nexus CMS - Main App
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './context/AuthContext';
import { Layout } from './components/Layout';
import { LoginPage } from './pages/LoginPage';
import { PagesListPage } from './pages/PagesListPage';
import { PageViewPage } from './pages/PageViewPage';
import { PageEditorPage } from './pages/PageEditorPage';
import { AdminPage } from './pages/AdminPage';
import { PortfolioPage } from './pages/PortfolioPage';
import { ProjectForm } from './components/ProjectForm';

// Protected Route wrapper
function ProtectedRoute({ children }) {
  const { user, loading } = useAuth();
  
  if (loading) return <div className="loading">Loading...</div>;
  if (!user) return <Navigate to="/login" replace />;
  
  return children;
}

// Public Route (redirect if logged in)
function PublicRoute({ children }) {
  const { user, loading } = useAuth();
  
  if (loading) return <div className="loading">Loading...</div>;
  if (user) return <Navigate to="/" replace />;
  
  return children;
}

function AppRoutes() {
  return (
    <Routes>
      {/* Public Routes */}
      <Route path="/" element={<Layout><PagesListPage /></Layout>} />
      <Route path="/page/:slug" element={<Layout><PageViewPage /></Layout>} />
      <Route path="/portfolio" element={<Layout><PortfolioPage /></Layout>} />
      <Route path="/login" element={<PublicRoute><LoginPage /></PublicRoute>} />
      
      {/* Protected Routes */}
      <Route path="/admin" element={<ProtectedRoute><Layout><AdminPage /></Layout></ProtectedRoute>} />
      <Route path="/admin/page/:slug" element={<ProtectedRoute><Layout><PageEditorPage /></Layout></ProtectedRoute>} />
      <Route path="/admin/project/new" element={<ProtectedRoute><Layout><ProjectForm /></Layout></ProtectedRoute>} />
      <Route path="/admin/project/:id/edit" element={<ProtectedRoute><Layout><ProjectForm /></Layout></ProtectedRoute>} />
      
      {/* 404 */}
      <Route path="*" element={<Layout><div className="not-found"><h1>404</h1><p>Page not found</p></div></Layout>} />
    </Routes>
  );
}

function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <AppRoutes />
      </AuthProvider>
    </BrowserRouter>
  );
}

export default App;