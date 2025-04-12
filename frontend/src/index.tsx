import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';
import reportWebVitals from './reportWebVitals';

// Log application startup
console.log('Minecraft Console application starting up');

const root = ReactDOM.createRoot(
	document.getElementById('root') as HTMLElement
);

// Important: We're not using StrictMode to prevent double-rendering
// which could cause duplicate WebSocket connections
root.render(
	<App />
);

// Log render complete
console.log('Root render complete');

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
