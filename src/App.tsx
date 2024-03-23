import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [awsProfiles, setAwsProfiles] = useState<string[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<string>("");
  const [identityInfo, setIdentityInfo] = useState<string>("");

  // Fetch AWS profiles when the component mounts
  useEffect(() => {
    const fetchProfiles = async () => {
      try {
        const profiles = await invoke("list_aws_profiles");
        // Asserting the profiles as an array of strings
        setAwsProfiles(profiles as string[]);
      } catch (error) {
        console.error("Failed to fetch AWS profiles:", error);
      }
    };

    fetchProfiles();
  }, []);

  const checkIdentity = async () => {
    try {
      const identity = await invoke("check_aws_identity", { profile: selectedProfile });
      // Asserting the identity information as a string
      setIdentityInfo(identity as string);
    } catch (error) {
      console.error("Failed to check AWS identity:", error);
    }
  };

  return (
    <div className="container">
      <h1>Welcome to AWS Credential Checker!</h1>

      {/* AWS Profile Selection */}
      <div>
        <label htmlFor="profile-select">Select AWS Profile:</label>
        <select
          id="profile-select"
          value={selectedProfile}
          onChange={(e) => setSelectedProfile(e.target.value)}
          disabled={awsProfiles.length === 0}
        >
          <option value="">--Please choose an AWS profile--</option>
          {awsProfiles.map((profile) => (
            <option key={profile} value={profile}>{profile}</option>
          ))}
        </select>
        <button onClick={checkIdentity} disabled={!selectedProfile}>Check</button>
      </div>

      {/* AWS Identity Information Display as TextArea */}
      <div>
        <h2>Identity Information:</h2>
        <textarea
          value={identityInfo}
          readOnly
          rows={10} // Adjust number of rows as needed
          cols={50} // Adjust width as needed
          style={{ resize: "none" }} // Prevent resizing
        ></textarea>
      </div>
    </div>
  );
}

export default App;
