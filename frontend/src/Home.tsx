import Navbar from "./Navbar";

function Home() {
    return (
        <div
            style={{
                backgroundImage: `url(violins.jpg)`,
                backgroundPosition: "center",
                backgroundRepeat: "no-repeat",
                backgroundSize: "cover",
                height: "100%",
            }}
        >
            <Navbar />
        </div>
    );
}

export default Home;
