import { useContext, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { AuthContext } from "./Auth";
import Navbar from "./Navbar";
import CredentialForm from "./CredentialForm";
import styles from "./css/Login.module.css";

function Login() {
    const [userName, setUserName] = useState("");
    const [password, setPassword] = useState("");
    const auth = useContext(AuthContext);
    const navigate = useNavigate();
    const location = useLocation();

    let from: string = location?.state?.from?.pathname || "/";

    const [message, setMessage] = useState(
        from === "/"
            ? "Please enter your login credentials"
            : "You must log in to view that page"
    );

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        if (userName === "") {
            alert("Username is required");
            return;
        }

        if (password === "") {
            alert("Password is required");
            return;
        }

        auth.login(userName, password, (loginSuccess: boolean) => {
            if (loginSuccess) {
                setMessage("Login successful, redirecting...");
                navigate(from, {
                    replace: true,
                });
            } else {
                setMessage("Unable to log in");
            }
        });
    };

    if (auth.getUser()) {
        return <div>You are already logged in</div>;
    }

    return (
        <div>
            <Navbar />
            <h1>Login</h1>
            <CredentialForm
                handleSubmit={handleSubmit}
                userName={userName}
                setUserName={setUserName}
                password={password}
                setPassword={setPassword}
            />
            <div className={styles.message}>{message}</div>
        </div>
    );
}

export default Login;
