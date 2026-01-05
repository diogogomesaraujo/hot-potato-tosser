<a id="readme-top"></a>


<!-- PROJECT LOGO -->
<br />
<div align="center">
    <img src="./assets/logo.png" alt="What is this" width="150">
  <h3 align="center">hot-potato-tosser</h3>
  <p align="center">
    An implementation of Mutual Exclusion with the Token Ring Algorithm in Rust.
  </p>
</div>


<!-- ABOUT THE PROJECT -->
## About

This project was developed as one of the assignments in the Distributed Systems class of my Master's degree at FCUP.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Assignment

- Create a ring network with 5 peers (call them p1 to p5) such as the one represented in the figure above. Each peer must be in a different machine (m1 to m5). Besides these 5 peers, create also a calculatormulti server called server that runs on machine m6. 

- Each peer only knows the IP address of the machine where the next peer is located: p1knows the IP of m2, p2 knows the IP of m3, . . . , p5 has the IP of m1, thus closing the ring. All peers know the IP address of the machine where the server is located (m6). These can be passed to the peer via the command line, e.g., for peer p2 you would run the following command in machine m2: 
  ```
  $ peer IP-of-m3 IP-of-m6
  ```

- One of the threads in each peer generates requests for the server following a Poisson distribution with a frequency of 4 per minute. The operation and the arguments for each request are also random. These requests are placed in a local queue.

- Another thread in that peer runs in a loop waiting for a message (that can only come from the previous peer). This message is designated a token. The token can be a void message. The peer that holds it at any given moment has exclusive access to server, effectively implementing mutual exclusion. 

- When the peer receives the token, it checks if it has requests for the server in its local queue. If it does, it holds the token until all requests are processed at the server; after the results are received and printed on the terminal, the peer restarts the forwarding of the token. If it does not, it forwards the token to the next peer in the ring. 

- **[EXTRA MARKS:]** Implement a scheme that enables the token to continue to move in the event of a failure in one of the peers.

<p align="right">(<a href="#readme-top">back to top</a>)</p>


### Built With

These are some of the tools used to build this project.

* `rand` for random number generation.
* `serde` for json serialization and deserialization.
* `tokio` for the TCP sockets implementation (multithreaded and async).
* `clap` for command-line argument passing.
* `color-print` for making it look nice.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

In order to run build this application from source you will need to have `cargo` installed. To install it you can follow this [tutorial](https://www.rust-lang.org/tools/install) from Rust's official website.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Installation
1. Clone the repository
   ```sh
   git clone https://github.com/diogogomesaraujo/hot-potato-tosser.git
   ```
2. Build the project
   ```sh
   cargo build
   ```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Run the Project
1. Navigate to the project's directory
2. Run the following command for all peers. Their connections should form a ring by making the last peer's next peer the first.
   
   ```sh
   cargo run --bin peer -- <OWN_ADDRESS> <SERVER_ADDRESS> <NEXT_PEER_ADDRESS>
   ```
3. Run the following command for the server.
   
   ```sh
   cargo run --bin server <SERVER_ADDRESS> <NUMBER_OF_PEERS>
   ```
#### Example

   ```sh
   # peers
   cargo run --bin peer -- 127.0.0.1:3001 127.0.0.1:3000 127.0.0.1:3002
   cargo run --bin peer -- 127.0.0.1:3002 127.0.0.1:3000 127.0.0.1:3003
   cargo run --bin peer -- 127.0.0.1:3003 127.0.0.1:3000 127.0.0.1:3001

   # server
   cargo run --bin server 127.0.0.1:3000 3
   ```


<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/othneildrew/Best-README-Template.svg?style=for-the-badge
[contributors-url]: https://github.com/othneildrew/Best-README-Template/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/othneildrew/Best-README-Template.svg?style=for-the-badge
[forks-url]: https://github.com/othneildrew/Best-README-Template/network/members
[stars-shield]: https://img.shields.io/github/stars/othneildrew/Best-README-Template.svg?style=for-the-badge
[stars-url]: https://github.com/othneildrew/Best-README-Template/stargazers
[issues-shield]: https://img.shields.io/github/issues/othneildrew/Best-README-Template.svg?style=for-the-badge
[issues-url]: https://github.com/othneildrew/Best-README-Template/issues
[license-shield]: https://img.shields.io/github/license/othneildrew/Best-README-Template.svg?style=for-the-badge
[license-url]: https://github.com/othneildrew/Best-README-Template/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/othneildrew
[product-screenshot]: images/screenshot.png
[Next.js]: https://img.shields.io/badge/next.js-000000?style=for-the-badge&logo=nextdotjs&logoColor=white
[Next-url]: https://nextjs.org/
[React.js]: https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB
[React-url]: https://reactjs.org/
[Vue.js]: https://img.shields.io/badge/Vue.js-35495E?style=for-the-badge&logo=vuedotjs&logoColor=4FC08D
[Vue-url]: https://vuejs.org/
[Angular.io]: https://img.shields.io/badge/Angular-DD0031?style=for-the-badge&logo=angular&logoColor=white
[Angular-url]: https://angular.io/
[Svelte.dev]: https://img.shields.io/badge/Svelte-4A4A55?style=for-the-badge&logo=svelte&logoColor=FF3E00
[Svelte-url]: https://svelte.dev/
[Laravel.com]: https://img.shields.io/badge/Laravel-FF2D20?style=for-the-badge&logo=laravel&logoColor=white
[Laravel-url]: https://laravel.com
[Bootstrap.com]: https://img.shields.io/badge/Bootstrap-563D7C?style=for-the-badge&logo=bootstrap&logoColor=white
[Bootstrap-url]: https://getbootstrap.com
[JQuery.com]: https://img.shields.io/badge/jQuery-0769AD?style=for-the-badge&logo=jquery&logoColor=white
[JQuery-url]: https://jquery.com
