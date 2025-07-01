import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import './styles.css';
import IndexPage from './components/IndexPage';
import EntryPage from './components/EntryPage';
import RedactPage from './components/RedactPage';

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<IndexPage />} />
        <Route path="/entry/:uuid" element={<EntryPage />} />
        <Route path="/redact" element={<RedactPage />} />
      </Routes>
    </Router>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
); 