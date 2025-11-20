import { useState, useEffect } from 'react';

export function useAuth() {
  const [user, setUser] = useState(null);
  useEffect(() => setUser({ name: 'tester' }), []);
  return user;
}

export function useWindowWidth() {
  const [width, setWidth] = useState(window.innerWidth);
  useEffect(() => {
    const handler = () => setWidth(window.innerWidth);
    window.addEventListener('resize', handler);
    return () => window.removeEventListener('resize', handler);
  }, []);
  return width;
}
