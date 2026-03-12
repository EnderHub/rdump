import React, { useState } from 'react';
import { Button } from './Button';
import useAuth from './useAuth';

// A simple component
function App() {
  const [count, setCount] = useState(0);
  const { user } = useAuth();

  return (
    <div>
      <h1>Welcome, {user?.name}</h1>
      <p>Count: {count}</p>
      <Button onClick={() => setCount(c => c + 1)} disabled={false} />
    </div>
  );
}
export default App;
