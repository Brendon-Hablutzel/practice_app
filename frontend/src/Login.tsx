import { useContext, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { AuthContext } from "./Auth";

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
        auth.login(userName, password, (loginSuccess: boolean) => {
            if (loginSuccess) {
                setMessage("Login successful, redirecting...");
                navigate(from, { replace: true });
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
            <h1>Login</h1>
            <h3>{message}</h3>
            <form onSubmit={handleSubmit}>
                <input
                    type="text"
                    placeholder="username"
                    value={userName}
                    onChange={(e) => setUserName(e.target.value)}
                />
                <input
                    type="password"
                    placeholder="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                />
                <input type="submit" />
            </form>
        </div>
    );
}

export default Login;
