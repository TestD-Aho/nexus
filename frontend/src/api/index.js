// API Service Layer
import axios from 'axios';

const API_URL = '/api/v1';

const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add auth token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('nexus_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle auth errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('nexus_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// ============ Auth ============
export const auth = {
  login: (email, password) => api.post('/auth/login', { email, password }),
  register: (data) => api.post('/auth/register', data),
  me: () => api.get('/auth/me'),
  refresh: (token) => api.post('/auth/refresh', token),
};

// ============ Pages ============
export const pages = {
  list: (params) => api.get('/pages', { params }),
  get: (slug) => api.get(`/pages/${slug}`),
  create: (data) => api.post('/pages', data),
  update: (id, data) => api.put(`/pages/${id}`, data),
  delete: (id) => api.delete(`/pages/${id}`),
};

// ============ Blocks ============
export const blocks = {
  list: (params) => api.get('/blocks', { params }),
  get: (id) => api.get(`/blocks/${id}`),
  create: (data) => api.post('/blocks', data),
  update: (id, data) => api.put(`/blocks/${id}`, data),
  delete: (id) => api.delete(`/blocks/${id}`),
  reorder: (blocks) => api.post('/blocks/reorder', { blocks }),
};

// ============ Collections ============
export const collections = {
  list: () => api.get('/collections'),
  get: (name) => api.get(`/collections/${name}`),
  create: (data) => api.post('/collections', data),
  createItem: (name, data) => api.post(`/collections/${name}/items`, data),
  updateItem: (name, id, data) => api.put(`/collections/${name}/items/${id}`, data),
  deleteItem: (name, id) => api.delete(`/collections/${name}/items/${id}`),
};

// ============ Media ============
export const media = {
  list: () => api.get('/media'),
  upload: (formData) => api.post('/media/upload', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  }),
  delete: (id) => api.delete(`/media/${id}`),
};

// ============ Admin ============
export const admin = {
  stats: () => api.get('/admin/stats'),
  users: () => api.get('/admin/users'),
  updateUser: (id, data) => api.put(`/admin/users/${id}`, data),
  roles: () => api.get('/admin/roles'),
  createRole: (data) => api.post('/admin/roles', data),
  permissions: () => api.get('/admin/permissions'),
};

// ============ System ============
export const system = {
  featureFlags: () => api.get('/system/feature-flags'),
  updateFeatureFlag: (key, enabled) => api.put(`/system/feature-flags/${key}`, { enabled }),
  maintenance: () => api.get('/system/maintenance'),
  setMaintenance: (enabled, message) => api.put('/system/maintenance', { enabled, message }),
};

export default api;
