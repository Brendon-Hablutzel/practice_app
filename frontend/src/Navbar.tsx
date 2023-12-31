import { useContext } from "react";
import { AuthContext } from "./Auth";
import { Link } from "react-router-dom";
import styles from "./css/Navbar.module.css";

function Navbar() {
    const auth = useContext(AuthContext);

    if (auth.getUser()) {
        return (
            <div className={styles.navbarWrapper}>
                <Link to="/">Home</Link>
                <Link to="/logout">Logout</Link>
                <Link to="/practice-sessions">
                    View and add practice sessions
                </Link>
                <Link to="/pieces">View and add pieces</Link>
            </div>
        );
    } else {
        return (
            <div className={styles.navbarWrapper}>
                <Link to="/">Home</Link>
                <Link to="/login">Login</Link>
                <Link to="/create-user">Create User</Link>
            </div>
        );
    }
}

export default Navbar;
