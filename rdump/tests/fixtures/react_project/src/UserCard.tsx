import React from 'react';

interface UserCardProps {
  name: string;
  email: string;
  avatar?: string;
}

export const UserCard: React.FC<UserCardProps> = ({ name, email, avatar }) => {
  return (
    <div className="user-card">
      {avatar && <img src={avatar} alt={name} />}
      <h2>{name}</h2>
      <p>{email}</p>
    </div>
  );
};

export default UserCard;
