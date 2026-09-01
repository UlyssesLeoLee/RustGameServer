import { NavLink } from 'react-router-dom';

interface Props {
  user: string;
  onLogout: () => void;
}

export default function NavBar({ user, onLogout }: Props) {
  return (
    <nav className="nav">
      <NavLink to="/dashboard" end>
        Dashboard
      </NavLink>
      <NavLink to="/players">Players</NavLink>
      <NavLink to="/servers">Servers</NavLink>
      <NavLink to="/items">Items</NavLink>
      <NavLink to="/canvas">Canvas</NavLink>
      <NavLink to="/mall">Mall</NavLink>
      <NavLink to="/reports">Reports</NavLink>
      <NavLink to="/support">Support</NavLink>
      <span className="nav-user">{user}</span>
      <button onClick={onLogout}>Logout</button>
    </nav>
  );
}
