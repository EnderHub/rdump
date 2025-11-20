import React, { useState, useEffect, useMemo } from 'react';

export function Button({ disabled }: { disabled?: boolean }) {
  const [count, setCount] = useState(0);
  useEffect(() => { console.log('effect'); }, []);
  const memoized = useMemo(() => count * 2, [count]);
  return <button disabled={disabled}>{memoized}</button>;
}
