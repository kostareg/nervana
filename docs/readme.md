# Nervana

Simulating the evolution of tiny neural networks.

## Nervana 2D

In this demo, 200 entities called "blobs" are placed in a 128x128 2D environment. Each one is wired with a randomly generated neural network. At the end of each generation, all blobs on the left half are removed (highlighted in red), and the remaining blobs are used to repopulate the next generation. This demonstration shows that as the generations progress, blobs gain the tendency to move towards the right, since that is the best method of survival per generation. A sample blob is highlighted in blue in each generation, and its neural network data is displayed on the user interface.

https://github.com/user-attachments/assets/16f84834-51f0-4f74-8cd2-63bffc20d8f4

## Nervana 3D
Using Rust's Bevy game engine to simulate thousands of small neural networks interacting and evolving in a 3 dimensional time-based environment. The following video shows a sample simulation where only entities in the top right survive. Notice how entities tend to learn to move to the top right over each generation, until eventually all survive.

https://github.com/user-attachments/assets/589054a6-0fa0-4360-a017-8195703b752a

## Analyzing neural networks

Since these neural networks are so small, we can attempt to analyze them using some visualization. The neural network of Nervana 2D's final sample in the video (from generation 10) is shown in Figure 1. Notably, the sum of the values inputted to Mx (at the bottom left) tend to be positive, so when they are plugged into `probability . tanh . sum` (the activation function for Mx), there is an increasing likelihood that the blob will move towards the right.

![neural network sample 10](https://github.com/user-attachments/assets/3c45e74c-63b5-47e8-b0f9-ae5b0c3e3c48)

Figure 1: neural network sample 10.

## Development

Dependencies for the visual tools are installed in the Nix shell. Run `nix develop` to enter. It's recommended to run the Bevy simulation in release mode if you have a restricted computer or no GPU.

## Next steps

I would love to implement rare mutations in order to increase survivorship in changing environments. In terms of simulation logic, I would like to implement collisions, killing neighbours, and pheremones. For data analysis, I want to create tools such as a streamlined neural network directed graph generator.
